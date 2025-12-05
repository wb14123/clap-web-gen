// ============================================================================
// Internationalization (i18n) Support
// ============================================================================

const translations = {
    en: {
        // Page UI
        language: 'Language',
        run: 'Run',
        reset: 'Reset',
        output: 'Output:',
        noOutputYet: 'No output yet. Fill in the form and click "Run".',

        // Subcommands
        subcommands: 'Subcommands',
        selectSubcommand: 'Select Subcommand',
        selectSubcommandPlaceholder: '-- Select a subcommand --',
        optionsFor: 'Options for',

        // Form fields
        selectOption: '-- Select an option --',
        enterValuePlaceholder: 'Enter value and press Enter',
        requiredField: 'Required field',

        // Status messages
        loadingWasm: 'Loading WASM module...',
        wasmLoaded: 'WASM module loaded successfully!',
        wasmLoadFailed: 'Failed to load WASM module: ',
        wasmNotReady: 'WASM module not ready yet. Please wait...',
        running: 'Running function...',
        success: 'Function executed successfully!',
        successNoReturn: 'Function executed successfully (no return value)',
        errorOccurred: 'Error occurred',
        fixValidationErrors: 'Please fix validation errors',
        validationError: 'Validation Error:',
        error: 'Error:',

        // Validation messages
        fieldRequired: 'Required field is empty',
        atLeastOneValue: 'At least one value is required',

        // Field help text
        flagRepeated: 'flag will be repeated N times',
    },
    zh: {
        // Page UI
        language: '语言',
        run: '运行',
        reset: '重置',
        output: '输出:',
        noOutputYet: '暂无输出。请填写表单并点击"运行"。',

        // Subcommands
        subcommands: '子命令',
        selectSubcommand: '选择子命令',
        selectSubcommandPlaceholder: '-- 选择一个子命令 --',
        optionsFor: '选项',

        // Form fields
        selectOption: '-- 选择一个选项 --',
        enterValuePlaceholder: '输入值并按回车',
        requiredField: '必填字段',

        // Status messages
        loadingWasm: '正在加载 WASM 模块...',
        wasmLoaded: 'WASM 模块加载成功！',
        wasmLoadFailed: 'WASM 模块加载失败: ',
        wasmNotReady: 'WASM 模块尚未准备就绪，请稍候...',
        running: '正在运行函数...',
        success: '函数执行成功！',
        successNoReturn: '函数执行成功（无返回值）',
        errorOccurred: '发生错误',
        fixValidationErrors: '请修复验证错误',
        validationError: '验证错误:',
        error: '错误:',

        // Validation messages
        fieldRequired: '必填字段为空',
        atLeastOneValue: '至少需要一个值',

        // Field help text
        flagRepeated: '标志将重复 N 次',
    }
};

// Get browser language, default to 'en'
function getBrowserLanguage() {
    const browserLang = navigator.language || navigator.userLanguage;
    // Check if it starts with 'zh' (Chinese)
    if (browserLang.startsWith('zh')) {
        return 'zh';
    }
    // Default to English
    return 'en';
}

// Current language (initialized from browser or localStorage)
let currentLanguage = localStorage.getItem('language') || getBrowserLanguage();

// Get translation for a key
function t(key) {
    return translations[currentLanguage]?.[key] || translations.en[key] || key;
}

// Set language and save to localStorage
function setLanguage(lang) {
    if (translations[lang]) {
        currentLanguage = lang;
        localStorage.setItem('language', lang);
        updatePageLanguage();
    }
}

// Update all translatable elements on the page
function updatePageLanguage() {
    // Update all elements with data-i18n attribute
    document.querySelectorAll('[data-i18n]').forEach(element => {
        const key = element.getAttribute('data-i18n');
        const translation = t(key);

        // Update text content or placeholder based on element type
        if (element.tagName === 'INPUT' || element.tagName === 'TEXTAREA') {
            if (element.hasAttribute('placeholder')) {
                element.placeholder = translation;
            }
        } else if (element.tagName === 'OPTION') {
            element.textContent = translation;
        } else {
            element.textContent = translation;
        }
    });

    // Update language selector
    const langSelector = document.getElementById('language-selector');
    if (langSelector) {
        langSelector.value = currentLanguage;
    }
}

// Initialize i18n when DOM is ready
function initI18n() {
    // Set up language selector
    const langSelector = document.getElementById('language-selector');
    if (langSelector) {
        langSelector.value = currentLanguage;
        langSelector.addEventListener('change', (e) => {
            setLanguage(e.target.value);
        });
    }

    // Apply initial translations
    updatePageLanguage();
}

// Export for use in other modules
window.i18n = {
    t,
    setLanguage,
    getCurrentLanguage: () => currentLanguage,
    initI18n,
    updatePageLanguage
};
