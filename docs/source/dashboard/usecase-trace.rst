Use Case Traceability
=====================

All Use Cases
-------------

.. needtable::
   :types: usecase
   :columns: id;title;status;links
   :style: datatables
   :sort: id

Use Case → Requirement Flows
-----------------------------

UC-TRANSFER-001 — Large File Transfer
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. needflow::
   :filter: id == 'UC-TRANSFER-001' or (type == 'req' and 'UC-TRANSFER-001' in links_back)
   :link_types: links
   :show_link_names:
   :show_legend:

UC-TRANSFER-002 — Large Directory Transfer
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. needflow::
   :filter: id == 'UC-TRANSFER-002' or (type == 'req' and 'UC-TRANSFER-002' in links_back)
   :link_types: links
   :show_link_names:
   :show_legend:

UC-TRANSFER-003 — Multi-USB Dataset Transfer
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. needflow::
   :filter: id == 'UC-TRANSFER-003' or (type == 'req' and 'UC-TRANSFER-003' in links_back)
   :link_types: links
   :show_link_names:
   :show_legend:

UC-TRANSFER-004 — SBOM Transfer
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. needflow::
   :filter: id == 'UC-TRANSFER-004' or (type == 'req' and 'UC-TRANSFER-004' in links_back)
   :link_types: links
   :show_link_names:
   :show_legend:

Requirements Without Use Case Coverage
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Functional requirements not referenced by any use case.  These are
typically cross-cutting concerns (CLI flags, error handling, safety
checks, deployment) or features covered by commands (``list``) that
lack a dedicated use case.

.. needtable::
   :filter: type == 'req' and len(links_back) == 0
   :columns: id;title;status;tags;release
   :style: datatables
   :sort: id
