module "terraform-aws-hasura" {
  source = "./modules/terraform-aws-hasura"

  region                   = var.region
  domain                   = var.domain
  hasura_subdomain         = var.hasura_subdomain
  app_subdomain            = var.app_subdomain
  hasura_version_tag       = var.hasura_version_tag
  hasura_admin_secret      = var.hasura_admin_secret
  hasura_jwt_secret_key    = var.hasura_jwt_secret_key
  rds_username             = var.rds_username
  rds_password             = var.rds_password
  rds_db_name              = var.rds_db_name
  rds_instance             = var.rds_instance
  vpc_enable_dns_hostnames = true
}
