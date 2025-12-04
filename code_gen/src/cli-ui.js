// ============================================================================
// WASM Module Initialization
// ============================================================================
import init, { [FUNCTION_NAME] } from '[IMPORT_PATH]';

let wasmReady = false;
const FIELDS = window.CLI_CONFIG.fields;
const SUBCOMMANDS = window.CLI_CONFIG.subcommands || [];
const form = document.getElementById('cliForm');
const output = document.getElementById('output');
const runButton = document.getElementById('runButton');
const wasmFunction = [FUNCTION_NAME];
let selectedSubcommand = null;

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
// Subcommand Handling
// ============================================================================
function initSubcommandSelector() {
    const selector = document.getElementById('subcommand-selector');
    if (!selector) return;

    selector.addEventListener('change', e => {
        const newSubcommand = e.target.value;
        selectedSubcommand = newSubcommand || null;

        // Hide all subcommand field sections and disable required validation
        document.querySelectorAll('.subcommand-fields').forEach(section => {
            section.style.display = 'none';
            // Disable HTML5 validation on hidden fields
            section.querySelectorAll('input[required], select[required]').forEach(input => {
                input.disabled = true;
            });
        });

        // Show the selected subcommand's fields and enable validation
        if (selectedSubcommand) {
            const section = document.getElementById(`subcommand-${selectedSubcommand}`);
            if (section) {
                section.style.display = 'block';
                // Re-enable HTML5 validation on visible fields
                section.querySelectorAll('input, select').forEach(input => {
                    input.disabled = false;
                });
            }
        }
    });

    // Initialize: disable all subcommand fields initially
    document.querySelectorAll('.subcommand-fields').forEach(section => {
        section.querySelectorAll('input, select').forEach(input => {
            input.disabled = true;
        });
    });
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

    // Helper to check if an element is in a visible section
    const isVisible = (element) => {
        // Check if element is in a subcommand section
        const subcommandSection = element.closest('.subcommand-fields');
        if (subcommandSection) {
            // Only validate if this subcommand section is visible
            return subcommandSection.style.display !== 'none';
        }
        // Otherwise it's a main field, always validate
        return true;
    };

    // Validate main command fields
    FIELDS.forEach(field => {
        const element = document.getElementById(field.name);
        if (!element) return;

        element.classList.remove('error');

        // Check required text/number fields
        if (field.required && field.field_type.type !== 'Bool' && field.field_type.type !== 'Vec') {
            if (!element.value.trim()) {
                const label = field.long || field.name;
                errors.push(`Field "${label}": Required field is empty`);
                element.classList.add('error');
            }
        }

        // Custom validation for Vec fields (not supported by HTML5)
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

    // Validate selected subcommand fields (if any)
    if (selectedSubcommand) {
        const subcommandConfig = SUBCOMMANDS.find(sc => sc.name === selectedSubcommand);
        if (subcommandConfig) {
            subcommandConfig.fields.forEach(field => {
                const elementId = `${selectedSubcommand}-${field.name}`;
                const element = document.getElementById(elementId);
                if (!element) return;

                element.classList.remove('error');

                // Check required text/number fields
                if (field.required && field.field_type.type !== 'Bool' && field.field_type.type !== 'Vec') {
                    if (!element.value.trim()) {
                        const label = field.long || field.name;
                        errors.push(`Field "${label}": Required field is empty`);
                        element.classList.add('error');
                    }
                }

                // Custom validation for Vec fields in subcommands
                if (field.field_type.type === 'Vec' && field.required) {
                    const values = getVecValues(elementId);
                    if (values.length === 0) {
                        const label = field.long || field.name;
                        errors.push(`Field "${label}": At least one value is required`);
                        const container = document.getElementById(`${elementId}-container`);
                        container.classList.add('error');
                    }
                }
            });
        }
    }

    return errors;
}

// ============================================================================
// CLI Argument Generation
// ============================================================================
function formToCliArgs() {
    const args = [];
    const positionalArgs = [];

    // Process main command fields
    FIELDS.forEach(field => {
        const element = document.getElementById(field.name);
        if (!element) return;

        if (field.is_positional) {
            // Store positional args for later (they go before subcommand)
            const value = element.value.trim();
            if (value) positionalArgs.push(value);
        } else {
            // Regular flag-based arguments
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
        }
    });

    // Add positional args before subcommand
    args.push(...positionalArgs);

    // Handle subcommand if one is selected
    if (selectedSubcommand) {
        args.push(selectedSubcommand);

        // Find the subcommand config
        const subcommandConfig = SUBCOMMANDS.find(sc => sc.name === selectedSubcommand);
        if (subcommandConfig) {
            const subPositionalArgs = [];

            subcommandConfig.fields.forEach(field => {
                const elementId = `${selectedSubcommand}-${field.name}`;
                const element = document.getElementById(elementId);
                if (!element) return;

                if (field.is_positional) {
                    // Positional args for subcommand
                    const value = element.value.trim();
                    if (value) subPositionalArgs.push(value);
                } else {
                    // Regular flag-based arguments for subcommand
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
                            getVecValues(elementId).forEach(value => {
                                args.push(flag, value);
                            });
                            break;

                        default: // String, Integer, Enum
                            const value = element.value.trim();
                            if (value) args.push(flag, value);
                    }
                }
            });

            // Add subcommand positional args at the end
            args.push(...subPositionalArgs);
        }
    }

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

    // Reset subcommand selection
    selectedSubcommand = null;
    const selector = document.getElementById('subcommand-selector');
    if (selector) {
        selector.value = '';
        // Hide all subcommand field sections and disable validation
        document.querySelectorAll('.subcommand-fields').forEach(section => {
            section.style.display = 'none';
            section.querySelectorAll('input, select').forEach(input => {
                input.disabled = true;
            });
        });
    }

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
initSubcommandSelector();
