"""Monkey-patch sphinx-needs' needpie charts for better legend placement.

The upstream ``process_needpie`` uses hardcoded ``figsize=(8, 4)`` and
``bbox_to_anchor=(0.8, 0, 0.5, 1)`` which causes the legend to overlap
pie labels when long category names are present.  This extension wraps
the original function and temporarily patches the matplotlib calls it
uses so the legend renders outside the axes area.
"""

from __future__ import annotations

from sphinx.application import Sphinx

# Upstream defaults we want to override
_ORIG_FIGSIZE = (8, 4)
_NEW_FIGSIZE = (10, 5)
_ORIG_BBOX = (0.8, 0, 0.5, 1)
_NEW_BBOX = (1.0, 0, 0.5, 1)
_SUBPLOTS_ADJUST_RIGHT = 0.7


def setup(app: Sphinx) -> dict:
    # Patch as early as possible — before any doctree processing.
    app.connect("builder-inited", _patch_needpie, priority=0)
    return {"version": "0.1", "parallel_read_safe": True}


def _patch_needpie(app: Sphinx) -> None:  # noqa: ARG001
    """Replace the process_needpie entry in NODE_TYPES with a wrapped version."""
    try:
        from sphinx_needs.directives.needpie import Needpie, process_needpie
        from sphinx_needs.needs import NODE_TYPES
    except ImportError:
        return

    if getattr(NODE_TYPES.get(Needpie), "_patched", False):
        return  # already patched

    _orig = process_needpie

    def _wrapped(app, doctree, fromdocname, found_nodes):
        import matplotlib.pyplot as plt
        from matplotlib.axes import Axes

        orig_subplots = plt.subplots
        orig_legend = Axes.legend

        def wider_subplots(*args, **kwargs):
            if kwargs.get("figsize") == _ORIG_FIGSIZE:
                kwargs["figsize"] = _NEW_FIGSIZE
            fig, ax = orig_subplots(*args, **kwargs)
            fig.subplots_adjust(right=_SUBPLOTS_ADJUST_RIGHT)
            return fig, ax

        def adjusted_legend(self, *args, **kwargs):
            if kwargs.get("bbox_to_anchor") == _ORIG_BBOX:
                kwargs["bbox_to_anchor"] = _NEW_BBOX
            return orig_legend(self, *args, **kwargs)

        plt.subplots = wider_subplots
        Axes.legend = adjusted_legend
        try:
            _orig(app, doctree, fromdocname, found_nodes)
        finally:
            plt.subplots = orig_subplots
            Axes.legend = orig_legend

    _wrapped._patched = True  # type: ignore[attr-defined]
    NODE_TYPES[Needpie] = _wrapped
