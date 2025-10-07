/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{vue,js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        
        'theme': {
          'sidebar-bg': '#2a1f21',
          'sidebar-text': '#f5e6e8',
          'sidebar-unread-text': '#ffffff',
          'sidebar-text-hover-bg': '#3d2b2e',
          'sidebar-text-active-border': '#ff7a8a',
          'sidebar-text-active-color': '#ffffff',
          'sidebar-header-bg': '#1f1617',
          'sidebar-header-text': '#f5e6e8',
          'sidebar-team-bar-bg': '#1f1617',
          'online-indicator': '#33d6a6',
          'away-indicator': '#f1c40f',
          'dnd-indicator': '#ec3750',
          'mention-bg': '#d63c56',
          'mention-color': '#ffffff',
          'center-channel-bg': '#3d2b2e',
          'center-channel-color': '#f5e6e8',
          'new-message-separator': '#ff7a8a',
          'link-color': '#ff9aaa',
          'button-bg': '#c8394f',
          'button-color': '#ffffff',
          'error-text': '#ff6b7d',
          'mention-highlight-bg': '#4a2d31',
          'mention-highlight-link': '#ff9aaa',
        },
        
        'bg-primary': 'var(--bg-primary)',
        'bg-secondary': 'var(--bg-secondary)',
        'bg-tertiary': 'var(--bg-tertiary)',
        'bg-card': 'var(--bg-card)',
        'bg-card-secondary': 'var(--bg-card-secondary)',
        'bg-card-tertiary': 'var(--bg-card-tertiary)',
        'bg-sidebar': 'var(--bg-sidebar)',
        'text-primary': 'var(--text-primary)',
        'text-secondary': 'var(--text-secondary)',
        'text-muted': 'var(--text-muted)',
        'accent-primary': 'var(--accent-primary)',
        'accent-secondary': 'var(--accent-secondary)',
        'accent-danger': 'var(--accent-danger)',
        'accent-warning': 'var(--accent-warning)',
        'accent-info': 'var(--accent-info)',
        'border-primary': 'var(--border-primary)',
        'border-secondary': 'var(--border-secondary)',
      },
      fontFamily: {
        'sans': ['"Outfit"', 'sans-serif'],
        'outfit': ['"Outfit"', 'sans-serif'],
      },
      borderRadius: {
        'xl': '12px',
        '2xl': '16px',
        '3xl': '20px',
      },
      boxShadow: {
        'primary': '0 4px 6px rgba(0, 0, 0, 0.2)',
        'secondary': '0 10px 25px rgba(0, 0, 0, 0.3)',
        'card': '0 2px 8px rgba(0, 0, 0, 0.1)',
        'card-hover': '0 4px 16px rgba(0, 0, 0, 0.15)',
      }
    },
  },
}
