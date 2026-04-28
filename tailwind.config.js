/** @type {import('tailwindcss').Config} */
export default {
  content: ['./src/**/*.{html,js,svelte,ts}'],
  theme: {
    extend: {
      colors: {
        brand: {
          green: 'var(--color-brand-green)',
          link: 'var(--color-brand-link)',
          'border-accent': 'var(--color-brand-border-accent)'
        },
        surface: {
          'near-black': '#0f0f0f',
          dark: '#171717',
          glass: 'rgba(41, 41, 41, 0.84)'
        },
        border: {
          dark: '#242424',
          std: '#2e2e2e',
          mid: '#363636',
          light: '#393939',
          charcoal: '#434343'
        },
        neutral: {
          dark: '#4d4d4d',
          mid: '#898989',
          light: '#b4b4b4',
          'near-white': '#efefef',
          'off-white': '#fafafa'
        },
        // Radix Color Tokens (HSL-based)
        radix: {
          violet: 'hsl(251, 63.2%, 63.2%)',
          slate: {
            5: 'var(--colors-slate5)',
            A12: 'var(--colors-slateA12)'
          }
          // Other radix colors can be added as needed or used via CSS variables
        }
      },
      fontFamily: {
        sans: ['Circular', 'Inter', 'Helvetica Neue', 'Helvetica', 'Arial', 'sans-serif'],
        mono: ['Source Code Pro', 'Office Code Pro', 'Menlo', 'monospace']
      },
      borderRadius: {
        std: '6px',
        comfortable: '8px',
        medium: '12px',
        large: '16px',
        pill: '9999px'
      },
      lineHeight: {
        tight: '1.00',
        section: '1.25',
        card: '1.33'
      },
      letterSpacing: {
        tight: '-0.16px',
        mono: '1.2px'
      },
      spacing: {
        '90': '90px',
        '96': '96px',
        '128': '128px'
      }
    }
  },
  plugins: []
};

