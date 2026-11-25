(function applyStoredTheme() {
  var preference = "system";
  try {
    preference = localStorage.getItem("fd-theme") || "system";
  } catch (error) {
    preference = "system";
  }
  var prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
  var dark = preference === "dark" || (preference === "system" && prefersDark);
  document.documentElement.dataset.theme = dark ? "dark" : "light";
})();
