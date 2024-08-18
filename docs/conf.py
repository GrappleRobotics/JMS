# Configuration file for the Sphinx documentation builder.

# -- Project information

project = 'JMS'
copyright = '2024, Grapple Robotics'
author = 'Grapple Robotics'

release = '2024'
version = open("../VERSION").read().strip()

# -- General configuration

extensions = [
    'sphinx.ext.duration',
    'sphinx.ext.doctest',
    'sphinx.ext.autodoc',
    'sphinx.ext.autosummary',
    'sphinx.ext.intersphinx',
]

intersphinx_mapping = {
    'python': ('https://docs.python.org/3/', None),
    'sphinx': ('https://www.sphinx-doc.org/en/master/', None),
}
intersphinx_disabled_domains = ['std']

templates_path = ['_templates']

# -- Options for HTML output

html_theme = 'sphinx_rtd_theme'

# -- Options for EPUB output
epub_show_urls = 'footnote'

rst_prolog = """
.. include:: /prolog.rst
"""