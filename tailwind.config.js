/** @type {import('tailwindcss').Config} */
module.exports = {
    content: [
        './templates/**/*.html',
        './core/forms.py',
    ],
    theme: {
        extend: {
            fontFamily: {
                'mouseMemoirs': ['Mouse Memoirs', 'cursive'],
            }
        },
    },
    plugins: [],
}

