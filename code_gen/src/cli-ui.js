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
// ANSI Color Code Parsing
// ============================================================================
function parseAnsiColors(text) {
    // Map ANSI color codes to CSS colors
    const colorMap = {
        // Standard colors (30-37)
        '30': '#000000',  // Black
        '31': '#cd3131',  // Red
        '32': '#0dbc79',  // Green
        '33': '#e5e510',  // Yellow
        '34': '#2472c8',  // Blue
        '35': '#bc3fbc',  // Magenta
        '36': '#11a8cd',  // Cyan
        '37': '#e5e5e5',  // White
        // Bright colors (90-97)
        '90': '#666666',  // Bright Black (Gray)
        '91': '#f14c4c',  // Bright Red
        '92': '#23d18b',  // Bright Green
        '93': '#f5f543',  // Bright Yellow
        '94': '#3b8eea',  // Bright Blue
        '95': '#d670d6',  // Bright Magenta
        '96': '#29b8db',  // Bright Cyan
        '97': '#ffffff',  // Bright White
    };

    const bgColorMap = {
        // Background colors (40-47)
        '40': '#000000',  // Black
        '41': '#cd3131',  // Red
        '42': '#0dbc79',  // Green
        '43': '#e5e510',  // Yellow
        '44': '#2472c8',  // Blue
        '45': '#bc3fbc',  // Magenta
        '46': '#11a8cd',  // Cyan
        '47': '#e5e5e5',  // White
        // Bright background colors (100-107)
        '100': '#666666',  // Bright Black
        '101': '#f14c4c',  // Bright Red
        '102': '#23d18b',  // Bright Green
        '103': '#f5f543',  // Bright Yellow
        '104': '#3b8eea',  // Bright Blue
        '105': '#d670d6',  // Bright Magenta
        '106': '#29b8db',  // Bright Cyan
        '107': '#ffffff',  // Bright White
    };

    // Handle both standard ANSI format (\x1b[XXm or \u001b[XXm) and simplified format ([XXm)
    // Match patterns like: \x1b[37m, \u001b[37m, or [37m
    const ansiRegex = /(?:\x1b|\u001b)?\[([0-9;]+)m/g;

    let currentColor = null;
    let currentBgColor = null;
    let isBold = false;
    let isItalic = false;
    let isUnderline = false;

    let result = '';
    let lastIndex = 0;
    let match;

    while ((match = ansiRegex.exec(text)) !== null) {
        // Add text before this code
        const textBefore = text.substring(lastIndex, match.index);
        if (textBefore) {
            if (currentColor || currentBgColor || isBold || isItalic || isUnderline) {
                const styles = [];
                if (currentColor) styles.push(`color: ${currentColor}`);
                if (currentBgColor) styles.push(`background-color: ${currentBgColor}`);
                if (isBold) styles.push('font-weight: bold');
                if (isItalic) styles.push('font-style: italic');
                if (isUnderline) styles.push('text-decoration: underline');

                result += `<span style="${styles.join('; ')}">${escapeHtml(textBefore)}</span>`;
            } else {
                result += escapeHtml(textBefore);
            }
        }

        // Parse the color code
        const codes = match[1].split(';');
        for (const code of codes) {
            if (code === '0') {
                // Reset all
                currentColor = null;
                currentBgColor = null;
                isBold = false;
                isItalic = false;
                isUnderline = false;
            } else if (code === '1') {
                isBold = true;
            } else if (code === '3') {
                isItalic = true;
            } else if (code === '4') {
                isUnderline = true;
            } else if (code === '22') {
                isBold = false;
            } else if (code === '23') {
                isItalic = false;
            } else if (code === '24') {
                isUnderline = false;
            } else if (colorMap[code]) {
                currentColor = colorMap[code];
            } else if (bgColorMap[code]) {
                currentBgColor = bgColorMap[code];
            }
        }

        lastIndex = ansiRegex.lastIndex;
    }

    // Add remaining text
    const remainingText = text.substring(lastIndex);
    if (remainingText) {
        if (currentColor || currentBgColor || isBold || isItalic || isUnderline) {
            const styles = [];
            if (currentColor) styles.push(`color: ${currentColor}`);
            if (currentBgColor) styles.push(`background-color: ${currentBgColor}`);
            if (isBold) styles.push('font-weight: bold');
            if (isItalic) styles.push('font-style: italic');
            if (isUnderline) styles.push('text-decoration: underline');

            result += `<span style="${styles.join('; ')}">${escapeHtml(remainingText)}</span>`;
        } else {
            result += escapeHtml(remainingText);
        }
    }

    return result || escapeHtml(text);
}

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
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
        output.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
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

        // Parse ANSI color codes and display with proper formatting
        if (result !== undefined && result !== null) {
            const resultText = typeof result === 'string' ? result : JSON.stringify(result, null, 2);
            output.innerHTML = parseAnsiColors(resultText);
        } else {
            output.textContent = 'Function executed successfully (no return value)';
        }

        setStatus('Function executed successfully!', 'success');
        setTimeout(() => setStatus('', ''), 2000);
        output.scrollIntoView({ behavior: 'smooth', block: 'nearest' });

    } catch (e) {
        output.className = 'error';
        output.textContent = 'Error:\n' + e;
        setStatus('Error occurred', 'error');
        output.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
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
