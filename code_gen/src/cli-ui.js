// ============================================================================
// WASM Module Initialization
// ============================================================================
import init, { [FUNCTION_NAME] } from '[IMPORT_PATH]';

let wasmReady = false;
const FIELDS = window.CLI_CONFIG.fields;
const form = document.getElementById('cliForm');
const output = document.getElementById('output');
const runButton = document.getElementById('runButton');
const wasmFunction = [FUNCTION_NAME];

function setStatus(message, type) {
    const statusDiv = document.getElementById('status');
    statusDiv.textContent = message;
    statusDiv.className = message ? `status ${type}` : '';
    statusDiv.style.display = message ? 'block' : 'none';
}

async function initWasm() {
    try {
        setStatus('Loading WASM module...', 'loading');
        await init();
        wasmReady = true;
        setStatus('WASM module loaded successfully!', 'success');
        setTimeout(() => setStatus('', ''), 2000);
    } catch (e) {
        setStatus('Failed to load WASM module: ' + e, 'error');
        console.error('Failed to load WASM module:', e);
    }
}

// ============================================================================
// Vec Field Handling (dynamic tag input)
// ============================================================================
function initVecFields() {
    document.querySelectorAll('.vec-input').forEach(input => {
        const fieldName = input.dataset.fieldName;
        const itemsContainer = document.getElementById(`${fieldName}-items`);

        input.addEventListener('keydown', e => {
            if (e.key === 'Enter' && input.value.trim()) {
                e.preventDefault();
                addVecItem(itemsContainer, input.value.trim());
                input.value = '';
            }
        });
    });
}

function addVecItem(container, value) {
    const item = document.createElement('div');
    item.className = 'vec-item';
    item.textContent = value;

    const removeBtn = document.createElement('span');
    removeBtn.className = 'vec-item-remove';
    removeBtn.textContent = '×';
    removeBtn.onclick = () => item.remove();

    item.appendChild(removeBtn);
    container.appendChild(item);
}

function getVecValues(fieldName) {
    const container = document.getElementById(`${fieldName}-items`);
    return Array.from(container.children).map(item =>
        item.textContent.slice(0, -1) // Remove the × character
    );
}

// ============================================================================
// Form Validation (using HTML5 + custom Vec validation)
// ============================================================================
function validateForm() {
    const errors = [];

    // Use native HTML5 validation first
    if (!form.checkValidity()) {
        const invalidInputs = form.querySelectorAll(':invalid');
        invalidInputs.forEach(input => {
            input.classList.add('error');
            const label = FIELDS.find(f => f.name === input.id)?.long || input.id;
            errors.push(`Field "${label}": Please fill out this field`);
        });
    }

    // Custom validation for Vec fields (not supported by HTML5)
    FIELDS.forEach(field => {
        if (field.field_type.type === 'Vec' && field.required) {
            const values = getVecValues(field.name);
            if (values.length === 0) {
                const label = field.long || field.name;
                errors.push(`Field "${label}": At least one value is required`);
                const container = document.getElementById(`${field.name}-container`);
                container.classList.add('error');
            }
        }
    });

    return errors;
}

// ============================================================================
// CLI Argument Generation
// ============================================================================
function formToCliArgs() {
    const args = [];

    FIELDS.forEach(field => {
        const element = document.getElementById(field.name);
        if (!element) return;

        const flag = field.long ? `--${field.long}` : `-${field.short}`;
        const fieldType = field.field_type.type;

        switch (fieldType) {
            case 'Bool':
                if (element.checked) args.push(flag);
                break;

            case 'Counter':
                const count = parseInt(element.value) || 0;
                for (let i = 0; i < count; i++) {
                    args.push(flag);
                }
                break;

            case 'Vec':
                getVecValues(field.name).forEach(value => {
                    args.push(flag, value);
                });
                break;

            default: // String, Integer, Enum
                const value = element.value.trim();
                if (value) args.push(flag, value);
        }
    });

    return args;
}

// ============================================================================
// Main Function Execution
// ============================================================================
function runFunction() {
    if (!wasmReady) {
        setStatus('WASM module not ready yet. Please wait...', 'error');
        return;
    }

    // Clear previous error styling
    form.querySelectorAll('.error').forEach(el => el.classList.remove('error'));

    // Validate form
    const validationErrors = validateForm();
    if (validationErrors.length > 0) {
        output.className = 'error';
        output.textContent = 'Validation Error:\n' + validationErrors.join('\n');
        setStatus('Please fix validation errors', 'error');
        return;
    }

    // Build CLI arguments and execute
    try {
        const args = formToCliArgs();
        console.log('CLI args:', args);

        runButton.disabled = true;
        setStatus('Running function...', 'loading');

        const result = wasmFunction(args);

        output.className = 'success';
        output.textContent = result !== undefined && result !== null
            ? (typeof result === 'string' ? result : JSON.stringify(result, null, 2))
            : 'Function executed successfully (no return value)';
        setStatus('Function executed successfully!', 'success');
        setTimeout(() => setStatus('', ''), 2000);

    } catch (e) {
        output.className = 'error';
        output.textContent = 'Error:\n' + e;
        setStatus('Error occurred', 'error');
    } finally {
        runButton.disabled = false;
    }
}

// ============================================================================
// Form Reset (using native HTML5 form.reset() + custom Vec cleanup)
// ============================================================================
function clearForm() {
    form.reset();

    // Clear Vec field items (not handled by form.reset())
    document.querySelectorAll('.vec-items').forEach(container => {
        container.innerHTML = '';
    });

    // Reset output
    output.textContent = 'No output yet. Fill in the form and click "Run Function".';
    output.className = '';
    setStatus('', '');

    // Clear error styling
    form.querySelectorAll('.error').forEach(el => el.classList.remove('error'));
}

// ============================================================================
// Initialization
// ============================================================================
runButton.addEventListener('click', runFunction);
document.getElementById('clearButton').addEventListener('click', clearForm);

initWasm();
initVecFields();
