// Theme detection and toggle - runs before body renders to prevent flash
(function() {
  "use strict";

  var STORAGE_KEY = "atuin-web-theme";

  function getPreferredTheme() {
    var stored = localStorage.getItem(STORAGE_KEY);
    if (stored) return stored;
    return window.matchMedia("(prefers-color-scheme: light)").matches ? "light" : "dark";
  }

  function setTheme(theme) {
    document.documentElement.setAttribute("data-bs-theme", theme);
    localStorage.setItem(STORAGE_KEY, theme);
    updateIcons(theme);
  }

  function updateIcons(theme) {
    var sun = document.getElementById("theme-icon-sun");
    var moon = document.getElementById("theme-icon-moon");
    if (sun && moon) {
      sun.style.display = theme === "dark" ? "none" : "block";
      moon.style.display = theme === "dark" ? "block" : "none";
    }
  }

  // Apply theme immediately to prevent flash
  setTheme(getPreferredTheme());

  // Expose toggle function globally
  window.toggleTheme = function() {
    var current = document.documentElement.getAttribute("data-bs-theme");
    setTheme(current === "dark" ? "light" : "dark");
  };

  // Update icons after DOM loads
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", function() {
      updateIcons(getPreferredTheme());
    });
  }

  // Listen for system theme changes
  window.matchMedia("(prefers-color-scheme: dark)").addEventListener("change", function(e) {
    if (!localStorage.getItem(STORAGE_KEY)) {
      setTheme(e.matches ? "dark" : "light");
    }
  });
})();
