import { convert } from "./transpile/wit2wadm_component.js";

const dropArea = document.getElementById("drop-area");
const fileInput = document.querySelector("input[type=file]");

dropArea.addEventListener("dragover", dragOverHandler);
dropArea.addEventListener("drop", dropHandler);
dropArea.addEventListener("click", () => fileInput.click());

function dragOverHandler(event) {
  event.preventDefault();
  event.stopPropagation();
  event.dataTransfer.dropEffect = "copy";
  dropArea.classList.add("highlight");
}

function dropHandler(event) {
  event.preventDefault();
  event.stopPropagation();
  dropArea.classList.remove("highlight");

  const file = event.dataTransfer.files[0];
  displayFileInfo(file);
}

fileInput.addEventListener("change", fileOnChangeHandler);

function fileOnChangeHandler(event) {
  const file = event.target.files[0];
  displayFileInfo(file);
}

function displayFileInfo(file) {
  if (file) {
    const reader = new FileReader();
    reader.readAsArrayBuffer(file);
    console.dir(file);

    reader.onload = function (event) {
      const name = document.getElementById("name").value;
      const description = document.getElementById("description").value;
      const version = document.getElementById("version").value || "v0.1.0";
      const image =
        document.getElementById("image").value ||
        "file:///path/to/" + file.name;
      const arrayBuffer = event.target.result;
      const manifest = convert.componentToWadm(
        arrayBuffer,
        name,
        description,
        version,
        image
      ); // Process the ArrayBuffer with your module function
      displayYAML(manifest);
    };
  }
}

function setupEventListeners() {
  const copyButton = document.getElementById("copy-button");
  if (copyButton) {
    copyButton.addEventListener("click", copyToClipboard);
  }
}

function displayYAML(yamlString) {
  const fileInfoElement = document.getElementById("file-info");
  // Now just updating the code part, button stays intact
  fileInfoElement.innerHTML = `<pre><code class="language-yaml">${escapeHTML(
    yamlString
  )}</code></pre><button id="copy-button">Copy Manifest to Clipboard</button>`;
  setupEventListeners(); // Set up event listeners after updating HTML
  Prism.highlightAll();
}

function copyToClipboard() {
  const yamlText = document.querySelector("#file-info code").innerText;
  navigator.clipboard
    .writeText(yamlText)
    .then(() => {
      console.log("YAML copied to clipboard!");
    })
    .catch((err) => {
      console.error("Failed to copy YAML: ", err);
      alert("Failed to copy YAML.");
    });
}

function escapeHTML(html) {
  var text = document.createTextNode(html);
  var p = document.createElement("p");
  p.appendChild(text);
  return p.innerHTML;
}
