"""Link integrity tests: internal cross-references and rendered diagrams."""

from playwright.sync_api import Page, expect


def test_index_internal_links_resolve(page: Page, base_url: str) -> None:
    """All internal links on the index page point to existing pages."""
    page.goto(base_url)
    links = page.locator("a.reference.internal")
    count = links.count()
    assert count > 0, "No internal links found on index page"

    broken: list[str] = []
    for i in range(min(count, 20)):  # Sample up to 20 links
        href = links.nth(i).get_attribute("href")
        if href and not href.startswith("#"):
            resp = page.request.get(f"{base_url}/{href}")
            if not resp.ok:
                broken.append(f"{href} -> {resp.status}")

    assert broken == [], f"Broken links: {broken}"


def test_srs_internal_links_resolve(page: Page, base_url: str) -> None:
    """Internal links on the SRS page point to existing pages."""
    page.goto(f"{base_url}/requirements/srs.html")
    # Sample a few content-area internal links (skip sidebar)
    content_links = page.locator(".wy-nav-content a.reference.internal")
    count = content_links.count()
    if count == 0:
        return  # No cross-references to test

    broken: list[str] = []
    for i in range(min(count, 10)):
        href = content_links.nth(i).get_attribute("href")
        if href and not href.startswith("#"):
            # Resolve relative URL
            url = href if href.startswith("http") else f"{base_url}/requirements/{href}"
            resp = page.request.get(url)
            if not resp.ok:
                broken.append(f"{href} -> {resp.status}")

    assert broken == [], f"Broken links on SRS page: {broken}"


def test_diagrams_render(page: Page, base_url: str) -> None:
    """Graphviz diagrams render as images, not broken placeholders."""
    page.goto(f"{base_url}/design/sdd.html")
    page.wait_for_load_state("networkidle")
    # Graphviz diagrams render as <img> in the _images/ directory or inline SVG
    images = page.locator("img[src*='_images/'], img[src*='graphviz-'], svg.graphviz")
    if images.count() > 0:
        first = images.first
        expect(first).to_be_visible()
        # For <img> elements, verify natural dimensions (not broken)
        tag = first.evaluate("el => el.tagName.toLowerCase()")
        if tag == "img":
            natural_width = first.evaluate("el => el.naturalWidth")
            assert natural_width > 0, "Diagram image failed to load (naturalWidth == 0)"


def test_no_broken_images_on_index(page: Page, base_url: str) -> None:
    """No broken images on the index page."""
    page.goto(base_url)
    page.wait_for_load_state("networkidle")
    images = page.locator("img")
    count = images.count()
    broken: list[str] = []
    for i in range(count):
        img = images.nth(i)
        src = img.get_attribute("src") or ""
        # Skip data: URIs (inline images are always valid)
        if src.startswith("data:"):
            continue
        natural_width = img.evaluate("el => el.naturalWidth")
        if natural_width == 0:
            broken.append(src)
    assert broken == [], f"Broken images: {broken}"
