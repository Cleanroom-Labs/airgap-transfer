"""
Sphinx configuration for the unified AirGap Transfer documentation site.

Imports shared theme configuration from the spec-docs submodule's common/
directory, then adds unified-site-specific settings (dashboard, implementation
mapping, API reference).
"""

import sys
import os

# Import shared config from the spec-docs submodule's common/
sys.path.insert(0, os.path.abspath('../spec-docs/common'))
from theme_config import *  # noqa: F401, F403

# Local extensions (e.g. needpie layout fix)
sys.path.insert(0, os.path.abspath('_ext'))
extensions.append('needpie_fix')
extensions.append('needflow_tree_fix')

project = 'AirGap Transfer'
copyright = '2026, Cleanroom Labs'
author = 'Cleanroom Labs'
version = get_docs_version()
release = get_docs_version()

# Same needs_types as spec docs
needs_types = make_needs_types('TRANSFER-')

# Show only the ID in :need: role references (not title)
needs_role_need_template = "{id}"

# Top-to-bottom needflow layout for consistent graph rendering
needs_graphviz_styles = {
    "default": {
        "graph": {"rankdir": "TB"},
        "node": {"margin": "0.21,0.11"},
        "edge": {"minlen": "2"},
    },
}

# -- sphinxcontrib-rust: deferred -------------------------------------------
# sphinxcontrib-rust only documents public items.  This is a binary crate so
# internal modules (chunker, verifier, etc.) aren't visible to rustdoc.
# API docs are deferred until the crate exposes a lib.rs or the extension
# supports --document-private-items.  The implementation/ pages provide
# module-level documentation in the meantime.

# Paths relative to docs/source/ → sibling spec-docs/common/
html_static_path = ['../spec-docs/common/sphinx/_static', '_static']
templates_path = ['../spec-docs/common/sphinx/_templates']
html_favicon = setup_project_favicon('AirGap Transfer', os.path.abspath('../spec-docs/common'))

html_title = 'AirGap Transfer'
html_context = {
    'display_github': True,
    'github_user': 'cleanroom-labs',
    'github_repo': 'airgap-transfer',
    'github_version': 'main',
    'conf_py_path': '/docs/source/',
}
setup_project_icon(project_name='AirGap Transfer', html_context_dict=html_context)
setup_standalone_docs(project_name='AirGap Transfer', html_context_dict=html_context)
setup_version_context(html_context)
