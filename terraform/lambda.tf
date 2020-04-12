locals {
  region      = "ap-southeast-2"
  stage_name  = "dev"
  lambda_name = "isostatus_fb_auth"
}
variable "accountId" {}


# Configure the AWS Provider
provider "aws" {
  version = "~> 2.0"
  region  = local.region
}

# Configure the Cloudflare provider.
# You may optionally use version directive to prevent breaking changes occurring unannounced.
provider "cloudflare" {
  version = "~> 2.0"
  email   = var.cloudflare_email
  api_key = var.cloudflare_api_key
}

# -----------------------------------------------------------------------------
# Configure API Gateway
# -----------------------------------------------------------------------------

resource "aws_api_gateway_account" "agw-account" {
  cloudwatch_role_arn = aws_iam_role.cloudwatch.arn
}

resource "aws_api_gateway_method_settings" "agw-settings" {
  rest_api_id = aws_api_gateway_rest_api.fb_auth.id
  stage_name  = aws_api_gateway_deployment.deployment.stage_name
  method_path = "${aws_api_gateway_resource.fb_auth.path_part}/${aws_api_gateway_method.fb_auth.http_method}"

  settings {
    metrics_enabled = true
    logging_level   = "INFO"
  }
}

resource "aws_api_gateway_deployment" "deployment" {
  depends_on  = [aws_api_gateway_integration.integration, aws_cloudwatch_log_group.fb_auth_agw]
  rest_api_id = aws_api_gateway_rest_api.fb_auth.id
  stage_name  = local.stage_name

  variables = {
    "force" = "1"
  }
}

resource "aws_vpc_endpoint" "fb_auth" {
  vpc_id              = module.terraform-aws-hasura.vpc.id
  service_name        = "com.amazonaws.${local.region}.execute-api"
  private_dns_enabled = true
  vpc_endpoint_type   = "Interface"

  subnet_ids         = module.terraform-aws-hasura.private_subnets.*.id
  security_group_ids = [module.terraform-aws-hasura.agw_security_group.id]
}

# API Gateway
resource "aws_api_gateway_rest_api" "fb_auth" {
  name = "${local.lambda_name}-agw-rest-api"

  policy = <<EOF
{
    "Version": "2012-10-17",
    "Statement": [
        {
            "Effect": "Allow",
            "Principal": "*",
            "Action": "execute-api:Invoke",
            "Resource": [
                "*"
            ]
        },
        {
            "Effect": "Deny",
            "Principal": "*",
            "Action": "execute-api:Invoke",
            "Resource": [
                "*"
            ],
            "Condition" : {
                "StringNotEquals": {
                    "aws:SourceVpce": "${aws_vpc_endpoint.fb_auth.id}"
                }
            }
        }
    ]
}
EOF

  endpoint_configuration {
    types = ["PRIVATE"]

    vpc_endpoint_ids = [aws_vpc_endpoint.fb_auth.id]
  }
}

resource "aws_api_gateway_resource" "fb_auth" {
  path_part   = local.lambda_name
  parent_id   = aws_api_gateway_rest_api.fb_auth.root_resource_id
  rest_api_id = aws_api_gateway_rest_api.fb_auth.id
}

resource "aws_api_gateway_method" "fb_auth" {
  rest_api_id   = aws_api_gateway_rest_api.fb_auth.id
  resource_id   = aws_api_gateway_resource.fb_auth.id
  http_method   = "POST"
  authorization = "NONE"
}

resource "aws_api_gateway_integration" "integration" {
  rest_api_id             = aws_api_gateway_rest_api.fb_auth.id
  resource_id             = aws_api_gateway_resource.fb_auth.id
  http_method             = aws_api_gateway_method.fb_auth.http_method
  integration_http_method = "POST"
  type                    = "AWS_PROXY"
  uri                     = aws_lambda_function.fb_auth.invoke_arn
}

# -----------------------------------------------------------------------------
# Create Lambda
# -----------------------------------------------------------------------------

resource "aws_lambda_permission" "apigw_lambda" {
  statement_id  = "AllowExecutionFromAPIGateway"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.fb_auth.function_name
  principal     = "apigateway.amazonaws.com"

  # More: http://docs.aws.amazon.com/apigateway/latest/developerguide/api-gateway-control-access-using-iam-policies-to-invoke-api.html
  source_arn = "arn:aws:execute-api:${local.region}:${var.accountId}:${aws_api_gateway_rest_api.fb_auth.id}/*/${aws_api_gateway_method.fb_auth.http_method}${aws_api_gateway_resource.fb_auth.path}"
}

resource "aws_lambda_function" "fb_auth" {
  filename      = "../rust-graphql-authoriser/rust.zip"
  function_name = local.lambda_name
  role          = aws_iam_role.cloudwatch.arn
  handler       = "main.main"

  # The filebase64sha256() function is available in Terraform 0.11.12 and later
  # For Terraform 0.11.11 and earlier, use the base64sha256() function and the file() function:
  # source_code_hash = "${base64sha256(file("lambda_function_payload.zip"))}"
  source_code_hash = filebase64sha256("../rust-graphql-authoriser/rust.zip")

  runtime = "provided"

  depends_on = [aws_iam_role_policy.cloudwatch, aws_cloudwatch_log_group.fb_auth_lambda]


  environment {
    variables = {
      FB_ACCESS_TOKEN     = var.facebook_access_token
      JWT_KEY             = var.hasura_jwt_secret_key
      HASURA_API_URL      = var.hasura_api_url
      HASURA_ADMIN_SECRET = var.hasura_admin_secret
    }
  }
}

# -----------------------------------------------------------------------------
# Configure IAM and Logging
# -----------------------------------------------------------------------------

resource "aws_iam_role" "cloudwatch" {
  name = "cloudwatch_global"

  assume_role_policy = <<EOF
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Sid": "",
      "Effect": "Allow",
      "Principal": {
        "Service": [
          "apigateway.amazonaws.com",
          "lambda.amazonaws.com"
        ]
      },
      "Action": "sts:AssumeRole"
    }
  ]
}
EOF
}

resource "aws_iam_role_policy" "cloudwatch" {
  name = "default"
  role = aws_iam_role.cloudwatch.id

  policy = <<EOF
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "logs:CreateLogGroup",
        "logs:CreateLogStream",
        "logs:DescribeLogGroups",
        "logs:DescribeLogStreams",
        "logs:PutLogEvents",
        "logs:GetLogEvents",
        "logs:FilterLogEvents"
      ],
      "Resource": "arn:aws:logs:*:*:*"
    }
  ]
}
EOF
}


# This is to optionally manage the CloudWatch Log Group for the Lambda Function.
# If skipping this resource configuration, also add "logs:CreateLogGroup" to the IAM policy below.
resource "aws_cloudwatch_log_group" "fb_auth_lambda" {
  name              = "/aws/lambda/${local.lambda_name}"
  retention_in_days = 14
}

resource "aws_cloudwatch_log_group" "fb_auth_agw" {
  name              = "/aws/agw/${aws_api_gateway_rest_api.fb_auth.id}/${local.stage_name}"
  retention_in_days = 14
}
