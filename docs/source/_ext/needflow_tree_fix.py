"""Fix sphinx-needs filter_by_tree depth tracking bug.

The upstream ``filter_by_tree`` uses ``roots.update()`` which overwrites
entries unconditionally.  When a node is reachable via multiple paths at
different depths, a later discovery at a higher depth can overwrite an
earlier discovery at a lower depth, causing the node to be incorrectly
excluded by the depth limit.

This extension monkey-patches ``filter_by_tree`` to only update a node's
depth when the new path is shorter.
"""

from __future__ import annotations

from typing import Literal

from sphinx.application import Sphinx

from sphinx_needs.config import LinkOptionsType
from sphinx_needs.views import NeedsView


def _filter_by_tree_fixed(
    needs_view: NeedsView,
    root_id: str,
    link_types: list[LinkOptionsType],
    direction: Literal["both", "incoming", "outgoing"],
    depth: int | None,
) -> NeedsView:
    """Fixed version that preserves minimum depth when a node is
    discovered via multiple paths."""
    if root_id not in needs_view:
        return needs_view.filter_ids([])
    roots: dict[str, tuple[int, object]] = {root_id: (0, needs_view[root_id])}
    link_prefixes = (
        ("_back",)
        if direction == "incoming"
        else ("",)
        if direction == "outgoing"
        else ("", "_back")
    )
    links_to_process = [
        link["option"] + d for link in link_types for d in link_prefixes
    ]

    need_ids: list[str] = []
    while roots:
        root_id, (root_depth, root) = roots.popitem()
        if root_id in need_ids:
            continue
        if depth is not None and root_depth > depth:
            continue
        need_ids.append(root_id)
        for link_type_name in links_to_process:
            for i in root.get(link_type_name, []):
                if i in needs_view:
                    new_depth = root_depth + 1
                    if i in roots:
                        existing_depth = roots[i][0]
                        if new_depth >= existing_depth:
                            continue  # keep the shorter path
                    roots[i] = (new_depth, needs_view[i])

    return needs_view.filter_ids(need_ids)


def setup(app: Sphinx) -> dict:
    app.connect("builder-inited", _patch_filter_by_tree, priority=0)
    return {"version": "0.1", "parallel_read_safe": True}


def _patch_filter_by_tree(app: Sphinx) -> None:  # noqa: ARG001
    import sphinx_needs.directives.needflow._shared as shared_mod
    import sphinx_needs.directives.needflow._graphviz as graphviz_mod

    if getattr(shared_mod.filter_by_tree, "_patched", False):
        return

    shared_mod.filter_by_tree = _filter_by_tree_fixed
    graphviz_mod.filter_by_tree = _filter_by_tree_fixed
    _filter_by_tree_fixed._patched = True  # type: ignore[attr-defined]

    # Patch plantuml too if available
    try:
        import sphinx_needs.directives.needflow._plantuml as plantuml_mod
        plantuml_mod.filter_by_tree = _filter_by_tree_fixed
    except (ImportError, AttributeError):
        pass
