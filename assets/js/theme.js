// Theme detection and toggle - runs before body renders to prevent flash
(function() {
  "use strict";

  var STORAGE_KEY = "atuin-web-theme";

  function getPreferredTheme() {
    return localStorage.getItem(STORAGE_KEY) || "system";
  }

  function resolveTheme(pref) {
    if (pref === "system") {
      return window.matchMedia("(prefers-color-scheme: light)").matches ? "light" : "dark";
    }
    return pref;
  }

  function applyTheme(pref) {
    localStorage.setItem(STORAGE_KEY, pref);
    document.documentElement.setAttribute("data-bs-theme", resolveTheme(pref));
    updateIcons(pref);
  }

  function updateIcons(pref) {
    var sun = document.getElementById("theme-icon-sun");
    var moon = document.getElementById("theme-icon-moon");
    var system = document.getElementById("theme-icon-system");
    if (sun && moon && system) {
      sun.style.display = pref === "light" ? "block" : "none";
      moon.style.display = pref === "dark" ? "block" : "none";
      system.style.display = pref === "system" ? "block" : "none";
    }
  }

  // Apply theme immediately to prevent flash
  applyTheme(getPreferredTheme());

  // Expose toggle function globally: dark → light → system → dark
  window.toggleTheme = function() {
    var pref = getPreferredTheme();
    var next = pref === "dark" ? "light" : pref === "light" ? "system" : "dark";
    applyTheme(next);
  };

  // Update icons after DOM loads
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", function() {
      updateIcons(getPreferredTheme());
    });
  }

  // Listen for system theme changes — only act when preference is "system"
  window.matchMedia("(prefers-color-scheme: dark)").addEventListener("change", function() {
    if (getPreferredTheme() === "system") {
      applyTheme("system");
    }
  });
})();
