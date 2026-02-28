Requirement Coverage
====================

Functional Requirements — Test Coverage
----------------------------------------

.. needtable::
   :types: req
   :columns: id;title;status;release;tests_back
   :style: datatables
   :sort: id

Non-Functional Requirements — Test Coverage
--------------------------------------------

.. needtable::
   :types: nfreq
   :columns: id;title;status;release;tests_back
   :style: datatables
   :sort: id

Requirement → Test Traceability Flows
--------------------------------------

Pack Operations
~~~~~~~~~~~~~~~

.. needflow::
   :filter: type in ['req', 'test'] and "pack" in tags and "unpack" not in tags
   :link_types: tests
   :show_link_names:
   :show_legend:

Unpack Operations
~~~~~~~~~~~~~~~~~

.. needflow::
   :filter: type in ['req', 'test'] and "unpack" in tags
   :link_types: tests
   :show_link_names:
   :show_legend:

List Operations
~~~~~~~~~~~~~~~

.. needflow::
   :filter: type in ['req', 'test'] and "list" in tags
   :link_types: tests
   :show_link_names:
   :show_legend:

Integrity, Verification & Crypto Agility
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. needflow::
   :filter: type in ['req', 'test'] and ("verification" in tags or "integrity" in tags or "crypto-agility" in tags)
   :link_types: tests
   :show_link_names:
   :show_legend:

CLI Interface
~~~~~~~~~~~~~

.. needflow::
   :filter: type in ['req', 'test'] and "cli" in tags
   :link_types: tests
   :show_link_names:
   :show_legend:

State, Safety, Error Handling & Deployment
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. needflow::
   :filter: type in ['req', 'test'] and ("state" in tags or "safety" in tags or "error-handling" in tags or "error" in tags or "deployment" in tags)
   :link_types: tests
   :show_link_names:
   :show_legend:

Non-Functional: Performance, Scalability & Reliability
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. needflow::
   :filter: (type == 'nfreq' or type == 'test') and ("performance" in tags or "scalability" in tags or "reliability" in tags)
   :link_types: tests
   :show_link_names:
   :show_legend:

Non-Functional: Usability, Maintainability & Portability
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. needflow::
   :filter: (type == 'nfreq' or type == 'test') and ("usability" in tags or "maintainability" in tags or "portability" in tags)
   :link_types: tests
   :show_link_names:
   :show_legend:

Non-Functional: Security, Privacy & Offline
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. needflow::
   :filter: (type == 'nfreq' or type == 'test') and ("security" in tags or "privacy" in tags or "offline" in tags)
   :link_types: tests
   :show_link_names:
   :show_legend:
