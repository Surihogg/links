export function createCleanupManager() {
  const fns = [];
  return {
    add(fn) {
      fns.push(fn);
    },
    run() {
      for (const fn of fns) {
        try { fn(); } catch {}
      }
      fns.length = 0;
    }
  };
}
