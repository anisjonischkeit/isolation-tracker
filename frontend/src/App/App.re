module Client = {
  /* Create an InMemoryCache */
  let inMemoryCache = ApolloInMemoryCache.createInMemoryCache();

  /* Create an HTTP Link */
  let httpLink = ApolloLinks.createHttpLink(~uri=Config.hasura_url, ());

  let instance =
    ReasonApollo.createApolloClient(~link=httpLink, ~cache=inMemoryCache, ());
};

[@react.component]
let make = () => {
  <ReasonApollo.Provider client=Client.instance>
    <FBLogin />
  </ReasonApollo.Provider>;
};