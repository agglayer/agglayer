// Inject a rustdoc link into the mdbook toolbar (right-hand icon row).
// The deployed site serves rustdoc under rustdoc/agglayer/ relative to the
// book root.  Use mdbook's path_to_root so the link works regardless of
// whether the site is hosted at domain root (Cloudflare previews) or under
// a sub-path (GitHub Pages).
(function () {
  var rightButtons = document.querySelector(".right-buttons");
  if (!rightButtons) return;

  var base = typeof path_to_root === "string" ? path_to_root : "";
  var link = document.createElement("a");
  link.href = base + "rustdoc/agglayer/";
  link.title = "Rust API Docs (rustdoc)";
  link.setAttribute("aria-label", "Rust API Docs");

  // Use an inline SVG matching the mdbook toolbar icon pattern (fa-code).
  var span = document.createElement("span");
  span.className = "fa-svg";
  span.innerHTML =
    '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 640 512">' +
    "<!--! Font Awesome Free 6.2.0 by @fontawesome - https://fontawesome.com " +
    "License - https://fontawesome.com/license/free " +
    "(Icons: CC BY 4.0, Fonts: SIL OFL 1.1, Code: MIT License) " +
    "Copyright 2022 Fonticons, Inc. -->" +
    '<path d="M392.8 1.2c-17-4.9-34.7 5-39.6 22l-128 448c-4.9 17 5 34.7 ' +
    "22 39.6s34.7-5 39.6-22l128-448c4.9-17-5-34.7-22-39.6zm80.6 120.1c-12.5 " +
    "12.5-12.5 32.8 0 45.3L562.7 256l-89.4 89.4c-12.5 12.5-12.5 32.8 0 " +
    "45.3s32.8 12.5 45.3 0l112-112c12.5-12.5 12.5-32.8 0-45.3l-112-112c-12.5" +
    "-12.5-32.8-12.5-45.3 0zm-306.7 0c-12.5-12.5-32.8-12.5-45.3 0l-112 " +
    "112c-12.5 12.5-12.5 32.8 0 45.3l112 112c12.5 12.5 32.8 12.5 45.3 " +
    "0s12.5-32.8 0-45.3L73.3 256l89.4-89.4c12.5-12.5 12.5-32.8 0-45.3z" +
    '"/></svg>';
  link.appendChild(span);

  rightButtons.insertBefore(link, rightButtons.firstChild);
})();
