(() => {
  window.addEventListener("load", function () {
    const rootDiv = document.getElementById("root");
    rootDiv.appendChild(createNameField());
    rootDiv.appendChild(createHostButton());
  });
})();

function createNameField() {
  const playerName = document.createElement("textarea");
  playerName.addEventListener("change", function (changeNameEvent) {
    localStorage.setItem("playerName", changeNameEvent.currentTarget.value);
  });
  return playerName;
}

function createHostButton() {
  const hostButton = document.createElement("button");
  hostButton.innerText = "host new websocket";
  hostButton.addEventListener("click", async function (clickEvent) {
    const playerName = localStorage.getItem("playerName");
    console.log(playerName);
    if (!playerName) {
      alert("please chose a name first");
      return;
    }
    const websocket = new WebSocket("ws://127.0.0.1:6969/test");
    // const response = await fetch("/host", {
    //   method: "POST",
    //   body: JSON.stringify({ playerName }),
    // });
  });
  return hostButton;
}
