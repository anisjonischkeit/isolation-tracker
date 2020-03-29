type loginState =
  | Loading
  | LoggedIn(Hello.authResponse)
  | LoginError(string)
  | LoggedOut
  | LogoutError(string);

[@react.component]
let make = () => {
  let (loginState, setLoginState) =
    React.useState(() => {
      let optAuthResponse = Hello.getLoginState("facebook");
      switch (optAuthResponse) {
      | Some(auth) => LoggedIn(auth)
      | None => LoggedOut
      };
    });

  let loginToFB = _e => {
    setLoginState(_ => Loading);
    Hello.login("facebook")
    ->Promise.get(r =>
        switch (r) {
        | Ok(res) => setLoginState(_ => LoggedIn(res.authResponse))
        | Error(e) => setLoginState(_ => LogoutError(e.error.message))
        }
      );
  };

  let logoutFB = _e => {
    setLoginState(_ => Loading);
    Hello.logout("facebook")
    ->Promise.get(r =>
        switch (r) {
        | Ok(res) => setLoginState(_ => LoggedOut)
        | Error(e) => setLoginState(_ => LogoutError(e.error.message))
        }
      );
  };

  let makeBtn = disabled =>
    <button disabled onClick=loginToFB>
      "login with facebook"->React.string
    </button>;

  let logoutBtn = <button onClick=logoutFB> "logout"->React.string </button>;

  switch (loginState) {
  | LoggedOut => makeBtn(false)
  | LogoutError(msg) =>
    [|("failed to logout: " ++ msg)->React.string, logoutBtn|]->React.array
  | Loading => makeBtn(true)
  | LoginError(msg) =>
    <div>
      [|("failed to login: " ++ msg)->React.string, makeBtn(false)|]
      ->React.array
    </div>
  | LoggedIn(authResponse) =>
    <div>
      [|
        {("logged in as" ++ authResponse.access_token)->React.string},
        logoutBtn,
      |]
      ->React.array
    </div>
  };
};