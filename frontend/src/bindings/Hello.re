type displayType = [ | `Popup | `Page | `None];

type authResponse = {
  client_id: string,
  access_token: string,
  token_type: string,
  expires_in: float,
  id_token: string,
  state: string,
  session_state: string,
  network: string,
  display: displayType,
  redirect_uri: string,
  scope: string,
  expires: float,
};

// INIT

type initOptions = {facebook: string};

[@bs.module "hellojs"] external init: initOptions => unit = "init";

// LOGIN

type loginSuccessResponse = {authResponse};

type authError = {message: string};
type loginErrorResponse = {error: authError};

[@bs.module "hellojs"]
external login:
  string => Promise.Js.t(loginSuccessResponse, loginErrorResponse) =
  "login";
let login = network => login(network)->Promise.Js.toResult;

[@bs.module "hellojs"]
external logout: string => Promise.Js.t(unit, loginErrorResponse) = "logout";
let logout = network => logout(network)->Promise.Js.toResult;

[@bs.module "hellojs"]
external getLoginState: string => Js.Nullable.t(authResponse) =
  "getAuthResponse";
let getLoginState: string => option(authResponse) =
  network => {
    Belt.Option.(
      getLoginState(network)
      ->Js.Nullable.toOption
      ->flatMap(session =>
          if (session.expires > Js.Date.now() /. 1000.0) {
            Some(session);
          } else {
            None;
          }
        )
    );
  };