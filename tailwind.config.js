/** @type {import('tailwindcss').Config} */
export default {
  content: ['./src/**/*.{html,js,svelte,ts}'],
  theme: {
    extend: {
      colors: {
        base: '#171717',
        surface: '#0f0f0f',
        border: {
          subtle: '#242424',
          std: '#2e2e2e',
          heavy: '#363636',
          accent: 'rgba(62, 207, 142, 0.3)'
        },
        brand: {
          green: '#3ecf8e',
          link: '#00c573'
        },
        text: {
          primary: '#fafafa',
          secondary: '#b4b4b4',
          muted: '#898989'
        }
      },
      fontFamily: {
        sans: ['Circular', 'Inter', '-apple-system', 'BlinkMacSystemFont', 'Segoe UI', 'Roboto', 'sans-serif'],
        mono: ['Source Code Pro', 'Menlo', 'monospace']
      }
    }
  },
  plugins: []
};
