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
    const fileInfoElement = document.getElementById("file-info");
    fileInfoElement.innerHTML = "";

    const fileNameDiv = document.createElement("div");
    fileNameDiv.className = "file-detail";
    fileNameDiv.innerHTML = `<i class="fas fa-file-alt"></i><span class="file-name">${file.name}</span>`;

    const fileSizeDiv = document.createElement("div");
    fileSizeDiv.className = "file-detail";
    fileSizeDiv.innerHTML = `<i class="fas fa-weight-hanging"></i> Size: ${file.size} bytes`;

    fileInfoElement.appendChild(fileNameDiv);
    fileInfoElement.appendChild(fileSizeDiv);

    const reader = new FileReader();
    reader.readAsArrayBuffer(file);

    reader.onload = function (event) {
      const arrayBuffer = event.target.result;
      const manifest = convert.componentToWadm(arrayBuffer); // Process the ArrayBuffer with your module function
      console.dir(manifest);
    };
  }
}
