const fileInput = document.querySelector('#file-input');
const chooseButton = document.querySelector('#choose-files');
const dropZone = document.querySelector('#drop-zone');
const queuePanel = document.querySelector('#queue-panel');
const fileQueue = document.querySelector('#file-queue');
const fileCount = document.querySelector('#file-count');
const clearButton = document.querySelector('#clear-files');
const toast = document.querySelector('#toast');
const uploadButton = document.querySelector('#upload-files');
const sharedFileList = document.querySelector('#shared-file-list');
const emptyState = document.querySelector('#empty-state');
const fileTotal = document.querySelector('.file-total');

let selectedFiles = [];
let toastTimeout;

function readableSize(bytes) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

function showToast(message) {
  toast.textContent = message;
  toast.classList.add('visible');
  window.clearTimeout(toastTimeout);
  toastTimeout = window.setTimeout(() => toast.classList.remove('visible'), 3600);
}

function renderFiles() {
  fileQueue.replaceChildren();
  fileCount.textContent = String(selectedFiles.length);
  queuePanel.hidden = selectedFiles.length === 0;
  uploadButton.disabled = selectedFiles.length === 0;

  selectedFiles.forEach((file) => {
    const row = document.createElement('li');
    row.className = 'queue-item';

    const glyph = document.createElement('span');
    glyph.className = 'file-glyph';
    glyph.textContent = file.name.split('.').pop().slice(0, 4) || 'FILE';

    const details = document.createElement('span');
    details.className = 'queue-meta';

    const name = document.createElement('span');
    name.className = 'queue-name';
    name.textContent = file.name;

    const size = document.createElement('span');
    size.className = 'queue-size';
    size.textContent = readableSize(file.size);

    details.append(name, size);
    row.append(glyph, details);
    fileQueue.append(row);
  });
}

function addFiles(fileList) {
  const incoming = Array.from(fileList);
  if (incoming.length === 0) return;

  selectedFiles = [...selectedFiles, ...incoming];
  renderFiles();
  showToast(`${incoming.length} file${incoming.length === 1 ? '' : 's'} selected.`);
  fileInput.value = '';
}

chooseButton.addEventListener('click', () => fileInput.click());
fileInput.addEventListener('change', (event) => addFiles(event.currentTarget.files));
clearButton.addEventListener('click', () => {
  selectedFiles = [];
  renderFiles();
});

dropZone.addEventListener('click', (event) => {
  if (event.target.closest('button')) return;
  fileInput.click();
});

dropZone.addEventListener('keydown', (event) => {
  if (event.key === 'Enter' || event.key === ' ') {
    event.preventDefault();
    fileInput.click();
  }
});

for (const eventName of ['dragenter', 'dragover']) {
  dropZone.addEventListener(eventName, (event) => {
    event.preventDefault();
    dropZone.classList.add('dragging');
  });
}

for (const eventName of ['dragleave', 'drop']) {
  dropZone.addEventListener(eventName, (event) => {
    event.preventDefault();
    dropZone.classList.remove('dragging');
  });
}

dropZone.addEventListener('drop', (event) => addFiles(event.dataTransfer.files));

async function refreshSharedFiles() {
  try {
    const response = await fetch('/api/files');
    if (!response.ok) throw new Error('Could not load shared files.');
    const files = await response.json();
    sharedFileList.replaceChildren();
    emptyState.hidden = files.length > 0;
    fileTotal.textContent = `${files.length} ITEM${files.length === 1 ? '' : 'S'}`;
    for (const file of files) {
      const item = document.createElement('li');
      item.className = 'shared-file-item';
      const details = document.createElement('span');
      details.className = 'shared-file-meta';
      const name = document.createElement('strong');
      name.textContent = file.name;
      const size = document.createElement('small');
      size.textContent = readableSize(file.size);
      details.append(name, size);
      const download = document.createElement('a');
      download.className = 'quiet-button';
      download.href = file.url;
      download.textContent = 'Download';
      download.setAttribute('aria-label', `Download ${file.name}`);
      item.append(details, download);
      sharedFileList.append(item);
    }
  } catch (error) {
    showToast(error.message);
  }
}

uploadButton.addEventListener('click', async () => {
  if (selectedFiles.length === 0) return;
  const formData = new FormData();
  selectedFiles.forEach((file) => formData.append('files', file, file.name));
  uploadButton.disabled = true;
  uploadButton.firstChild.textContent = 'Uploading… ';
  try {
    const response = await fetch('/api/files', { method: 'POST', body: formData });
    if (!response.ok) throw new Error(await response.text() || 'Upload failed.');
    selectedFiles = [];
    renderFiles();
    showToast('Files uploaded and ready to share.');
    await refreshSharedFiles();
  } catch (error) {
    showToast(error.message);
  } finally {
    uploadButton.firstChild.textContent = 'Upload files ';
    uploadButton.disabled = selectedFiles.length === 0;
  }
});

refreshSharedFiles();
