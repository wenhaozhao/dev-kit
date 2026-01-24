import { ref, watchEffect } from 'vue';

const theme = ref(localStorage.getItem('theme') || 'auto');

export function useTheme() {
  const toggleTheme = () => {
    if (theme.value === 'dark') {
      theme.value = 'light';
    } else {
      theme.value = 'dark';
    }
  };

  watchEffect(() => {
    localStorage.setItem('theme', theme.value);
    const isDark = theme.value === 'dark' || 
      (theme.value === 'auto' && window.matchMedia('(prefers-color-scheme: dark)').matches);
    
    if (isDark) {
      document.documentElement.classList.add('dark-mode');
    } else {
      document.documentElement.classList.remove('dark-mode');
    }
  });

  return {
    theme,
    toggleTheme
  };
}
