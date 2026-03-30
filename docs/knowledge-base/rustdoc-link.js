// Inject a rustdoc link into the mdbook toolbar (right-hand icon row).
// The deployed site serves rustdoc at /rustdoc/agglayer/.
(function () {
  var rightButtons = document.querySelector(".right-buttons");
  if (!rightButtons) return;

  var link = document.createElement("a");
  link.href = "/rustdoc/agglayer/";
  link.title = "Rust API Docs (rustdoc)";
  link.setAttribute("aria-label", "Rust API Docs");

  var icon = document.createElement("i");
  icon.className = "fa fa-code";
  link.appendChild(icon);

  rightButtons.insertBefore(link, rightButtons.firstChild);
})();
