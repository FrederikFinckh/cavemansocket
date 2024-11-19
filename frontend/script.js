(() => {
  window.addEventListener("load", function () {
    const rootDiv = document.getElementById("root");
    rootDiv.appendChild(createNameField());
    rootDiv.appendChild(document.createElement("br"));
    rootDiv.appendChild(createHostButton());
    rootDiv.appendChild(createSessionsList());
    updateSessions();
  });
})();

const SESSION_LIST_ID = "session-list";

function createSessionsList() {
  const session_container = document.createElement("div");
  const headline = document.createElement("h4");
  headline.innerText = "Open sessions:";
  session_container.appendChild(headline);
  const list = document.createElement("ul");
  session_container.appendChild(list);
  list.setAttribute("id", SESSION_LIST_ID);
  return session_container;
}

async function updateSessions() {
  let list = document.getElementById(SESSION_LIST_ID);
  fetch("/sessions")
    .then((response) => response.json())
    .then((sessions) => {
      list.innerHTML = "";
      sessions.forEach((session) => {
        const entry = document.createElement("li");
        entry.innerText = JSON.stringify(session);
        list.appendChild(entry);
      });
    });
}

function createNameField() {
  const username = document.createElement("textarea");
  username.addEventListener("change", function (changeNameEvent) {
    localStorage.setItem("username", changeNameEvent.currentTarget.value);
  });
  return username;
}

function createHostButton() {
  const hostButton = document.createElement("button");
  hostButton.innerText = "host new websocket";
  hostButton.addEventListener("click", async function () {
    const username = localStorage.getItem("username");
    console.log(username);
    if (!username) {
      alert("please chose a name first");
      return;
    }
    await fetch("/host", {
      method: "POST",
      body: JSON.stringify({ username }),
    })
      .then((response) => response.json())
      .then((data) => {
        console.log(`received response: ${data}`);
        updateSessions();
      });
  });
  return hostButton;
}
