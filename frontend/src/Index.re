// Entry point

[@bs.val] external document: Js.t({..}) = "document";

Hello.init({facebook: "343367289954741"});

let content = document##createElement("div");

content##className #= "content";
document##body##appendChild(content);

ReactDOMRe.render(<App />, content);