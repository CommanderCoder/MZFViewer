// Import the WASM module generated by wasm-pack.
// The path './pkg/rust_wasm_converter.js' assumes that wasm-pack
// builds into a 'pkg' directory relative to this HTML file.
import init, { process_binary } from './pkg/rust_wasm_converter.js';

// Utility to get query string parameters
function getQueryParam(name) {
    const urlParams = new URLSearchParams(window.location.search);
    return urlParams.get(name);
}


// Get references to DOM elements
const fileInput = document.getElementById('fileInput');
const outputPre = document.getElementById('output');
const modeSA = document.getElementById('modeSA');
const modeSP = document.getElementById('modeSP');
const mode1Z = document.getElementById('mode1Z');
const modeZ80 = document.getElementById('modeZ80');
const modeZX80Basic = document.getElementById('modeZX80Basic');
const modeDump = document.getElementById('modeDump');
const messageParagraph = document.getElementById('message');
const useAltParagraph = document.getElementById('useAlt');
const saveButton = document.getElementById('saveButton');
const charset = document.getElementById('charset');
const charsetToggle = document.getElementById('charsetToggle');
const charsetLabel = document.querySelector('label[for="charsetToggle"]');
const fileInputSection = fileInput.closest('div'); // The file upload section container
const outputTypeSpan = document.getElementById('outputType');
const titleElement = document.getElementById('title');
const divSA = document.getElementById('divSA');
const divSP = document.getElementById('divSP');
const div1Z = document.getElementById('div1Z');
const divZ80 = document.getElementById('divZ80');
const mzbyte0 = document.getElementById('mzbyte0');
const divZX80 = document.getElementById('divZX80');

// Initialize fileData and fileName
let fileData = null;
let fileName = 'MZFBasic';

// Function to enable/disable the save button
const toggleSaveButton = () => {
    saveButton.disabled = outputPre.textContent.trim() === '' || fileData === null;
};

// Function to process the file data based on the selected mode
let processFile; // Declare here for use in fetch logic

// Check for URL in query string
const fileUrl = getQueryParam('url');

// check for mode in query string
const modeParam = getQueryParam('mode');
if (modeParam) {
    if (modeParam === 'SA') modeSA.checked = true;
    else if (modeParam === 'SP') modeSP.checked = true;
    else if (modeParam === '1Z') mode1Z.checked = true;
    else if (modeParam === 'Z80') modeZ80.checked = true;   
    else if (modeParam === 'DUMP') modeDump.checked = true;
    else if (modeParam === 'ZX80BASIC') modeZX80Basic.checked = true;
    else modeDump.checked = true; // Default to DUMP if invalid mode
} else {
    // This is a placeholder for `shared.js`. The actual default will be set in the specific HTML files.
    // For MZF viewer (index.html), modeSP.checked = true;
    // For ZX viewer (index2.html), modeZX80Basic.checked = true;
}

// Initialize the WASM module. This must be called before using any WASM functions.
init().then(() => {
    console.log("WASM module initialized successfully.");

    processFile = () => {
        if(divZX80 && divZX80.classList.contains('hidden')) { // Check if divZX80 exists and is hidden
            if (modeDump.checked || modeZ80.checked) {
                charset.classList.remove('hidden');
            } else {
                charset.classList.add('hidden');
            }
        } else { // For ZX viewer, charset is always available if it exists
            if (charset) charset.classList.remove('hidden');
        }


        if (!fileData) {
            outputPre.textContent = '';
            messageParagraph.classList.remove('hidden');
            useAltParagraph.classList.add('hidden');
            toggleSaveButton();
            fileName = 'MZFBasic';
            return;
        }

        fileName = fileData.fileName || 'MZFBasic';
        messageParagraph.classList.add('hidden');
        useAltParagraph.classList.remove('hidden');

        let mode;
        if (modeSA && modeSA.checked) mode = 'SA';
        else if (modeSP && modeSP.checked) mode = 'SP';
        else if (mode1Z && mode1Z.checked) mode = '1Z';
        else if (modeZ80 && modeZ80.checked) mode = 'Z80';
        else if (modeDump && modeDump.checked) mode = 'DUMP';
        else if (modeZX80Basic && modeZX80Basic.checked) mode = 'ZX80BASIC';
        else mode = 'SA'; // Fallback for MZF viewer, or will be overridden by specific HTML

        // Only try to read the first byte and set outputTypeSpan if mzbyte0 element exists
        if (mzbyte0 && !mzbyte0.classList.contains('hidden')) {
            try {
                const firstbyte = new Uint8Array(fileData)[0];
                const typeText = {
                    0x01: 'Machine Code (Z80)',
                    0x02: 'BASIC (SP-5025) or BASIC (SA-5510)',
                    0x05: 'BASIC (1Z-013B)',
                    // Add more types as needed
                }[firstbyte] || 'Unknown Type';
                outputTypeSpan.textContent = `0x${firstbyte.toString(16).padStart(2, '0')} - ${typeText}`;
            } catch (e) {
                console.error("Error reading first byte:", e);
            }
        }

        try {
            const result = process_binary(new Uint8Array(fileData), mode, charsetToggle.checked);
            outputPre.textContent = result;
        } catch (e) {
            console.error("Error processing binary:", e);
            outputPre.textContent = `Error: Could not process file. ${e.message || e}`;
        } finally {
            toggleSaveButton();
        }
    };

    if (fileUrl) {
        // Hide file input section if using URL
        if (fileInputSection) fileInputSection.style.display = 'none';

        const handleZipFromBuffer = async (arrayBuffer, zipFileName) => {
            try {
                const zip = await JSZip.loadAsync(arrayBuffer);
                let targetFile = null;
                let fileExtension = /\.mzf$/i;

                // Determine target file extension based on current viewer (MZF or ZX)
                if (titleElement && titleElement.textContent === 'Sinclair ZX File Viewer') {
                    fileExtension = /\.tap$/i; // Assuming .tap for ZX files
                }

                zip.forEach((relativePath, zipEntry) => {
                    if (!targetFile && fileExtension.test(zipEntry.name)) {
                        targetFile = zipEntry;
                    }
                });

                if (!targetFile) {
                    outputPre.textContent = `Error: No ${fileExtension.source.replace(/\\|\^|\$/g, '')} file found in ZIP archive.`;
                    fileData = null;
                    toggleSaveButton();
                    return;
                }
                const targetBuffer = await targetFile.async('arraybuffer');
                fileData = targetBuffer;
                fileData.fileName = targetFile.name.replace(/\.[^/.]+$/, "");
                processFile();
            } catch (err) {
                outputPre.textContent = `Error: Failed to extract file from ZIP. ${err.message || err}`;
                fileData = null;
                toggleSaveButton();
            }
        };

        fetch("https://corsproxy.io/?"+fileUrl)
            .then(async response => {
                if (!response.ok) throw new Error(`HTTP error ${response.status}`);
                const arrayBuffer = await response.arrayBuffer();
                // Check if URL ends with .zip (case-insensitive)
                if (/\.zip$/i.test(fileUrl)) {
                    await handleZipFromBuffer(arrayBuffer, fileUrl);
                } else {
                    fileData = arrayBuffer;
                    // Try to extract filename from URL
                    try {
                        const urlObj = new URL(fileUrl);
                        const urlPath = urlObj.pathname.split('/');
                        const urlFile = urlPath[urlPath.length - 1] || 'File';
                        fileData.fileName = urlFile.replace(/\.[^/.]+$/, "");
                    } catch {
                        fileData.fileName = 'File';
                    }
                    processFile();
                }
            })
            .catch(e => {
                outputPre.textContent = `Error: Could not fetch file from URL. ${e.message || e}`;
                fileData = null;
                toggleSaveButton();
            });

        // Disable file input events
        fileInput.disabled = true;
    } else {
        // Normal file input mode
        fileInput.addEventListener('change', (event) => {
            const file = event.target.files[0];
            if (file) {
                if (file.name.toLowerCase().endsWith('.zip')) {
                    // Handle ZIP file
                    const reader = new FileReader();
                    reader.onload = async (e) => {
                        try {
                            const zip = await JSZip.loadAsync(e.target.result);
                            let targetFile = null;
                            let fileExtension = /\.mzf$/i;

                            // Determine target file extension based on current viewer (MZF or ZX)
                            if (titleElement && titleElement.textContent === 'Sinclair ZX File Viewer') {
                                fileExtension = /\.tap$/i; // Assuming .tap for ZX files
                            }

                            zip.forEach((relativePath, zipEntry) => {
                                if (!targetFile && fileExtension.test(zipEntry.name)) {
                                    targetFile = zipEntry;
                                }
                            });

                            if (!targetFile) {
                                outputPre.textContent = `Error: No ${fileExtension.source.replace(/\\|\^|\$/g, '')} file found in ZIP archive.`;
                                fileData = null;
                                toggleSaveButton();
                                return;
                            }
                            // Extract as ArrayBuffer
                            const targetBuffer = await targetFile.async('arraybuffer');
                            fileData = targetBuffer;
                            fileData.fileName = targetFile.name.replace(/\.[^/.]+$/, "");
                            processFile();
                        } catch (err) {
                            outputPre.textContent = `Error: Failed to extract file from ZIP. ${err.message || err}`;
                            fileData = null;
                            toggleSaveButton();
                        }
                    };
                    reader.readAsArrayBuffer(file);
                } else {
                    // Handle normal file
                    const reader = new FileReader();
                    reader.onload = (e) => {
                        fileData = e.target.result;
                        fileData.fileName = file.name.replace(/\.[^/.]+$/, "");
                        processFile();
                    };
                    reader.readAsArrayBuffer(file);
                }
            } else {
                fileData = null;
                outputPre.textContent = '';
                messageParagraph.classList.remove('hidden');
                useAltParagraph.classList.add('hidden');
                toggleSaveButton();
            }
        });
    }

    // Event listeners for radio button changes
    if (modeSA) modeSA.addEventListener('change', () => processFile && processFile());
    if (modeSP) modeSP.addEventListener('change', () => processFile && processFile());
    if (mode1Z) mode1Z.addEventListener('change', () => processFile && processFile());
    if (modeZ80) modeZ80.addEventListener('change', () => processFile && processFile());
    if (modeDump) modeDump.addEventListener('change', () => processFile && processFile());
    if (modeZX80Basic) modeZX80Basic.addEventListener('change', () => processFile && processFile());
    if (charsetToggle) charsetToggle.addEventListener('change', () => processFile && processFile());
    
    // Event listener for the Save button
    saveButton.addEventListener('click', () => {
        const textToSave = outputPre.textContent;
        if (textToSave.trim() === '') {
            console.warn("No content to save.");
            return;
        }
        const blob = new Blob([textToSave], { type: 'text/plain;charset=utf-8' });
        const url = URL.createObjectURL(blob);
        let filename = prompt("Enter a filename for the output:", `${fileName}.txt`);
        if (!filename) {
            URL.revokeObjectURL(url);
            return;
        }
        const a = document.createElement('a');
        a.href = url;
        a.download = filename;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
    });

    // Initial call to hide the message and disable save button if no file is selected on load
    if (!fileUrl) processFile();
}).catch(e => {
    console.error("Failed to initialize WASM module:", e);
    outputPre.textContent = `Error: Failed to load WASM module. Please check console for details.`;
    messageParagraph.classList.add('hidden');
    toggleSaveButton();
});