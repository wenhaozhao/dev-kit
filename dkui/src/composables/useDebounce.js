import { ref } from "vue";

export function useDebounce() {
  let debounceTimer = null;
  function debounce(fn, delay = 300) {
    return (...args) => {
      clearTimeout(debounceTimer);
      debounceTimer = setTimeout(() => fn(...args), delay);
    };
  }
  return { debounce };
}
