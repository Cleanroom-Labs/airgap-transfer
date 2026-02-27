"""Navigation tests: sidebar, breadcrumbs, search, and page sequencing."""

import re

from playwright.sync_api import Page, expect


def test_sidebar_has_expected_sections(page: Page, base_url: str) -> None:
    """Sidebar contains the four top-level caption headings."""
    page.goto(base_url)
    sidebar = page.locator(".wy-menu-vertical")
    for section in ["Dashboard", "Specification", "Planning", "Implementation"]:
        caption = sidebar.locator(f".caption-text:text-is('{section}')")
        expect(caption).to_be_visible()


def test_sidebar_link_navigates(page: Page, base_url: str) -> None:
    """Clicking a sidebar link loads the target page."""
    page.goto(base_url)
    link = page.locator(".wy-menu-vertical a.reference.internal", has_text="Project Health").first
    link.click()
    expect(page).to_have_url(re.compile(r"dashboard/proj-health"))


def test_breadcrumbs_visible(page: Page, base_url: str) -> None:
    """Breadcrumbs are rendered on a sub-page."""
    page.goto(f"{base_url}/dashboard/proj-health.html")
    breadcrumbs = page.locator(".wy-breadcrumbs")
    expect(breadcrumbs).to_be_visible()
    # Should contain at least "Docs" (home) and "Project Health"
    count = breadcrumbs.locator("li").count()
    assert count > 1, f"Expected more than 1 breadcrumb item, got {count}"


def test_breadcrumb_home_link(page: Page, base_url: str) -> None:
    """The first breadcrumb links back to the index."""
    page.goto(f"{base_url}/requirements/srs.html")
    home_link = page.locator(".wy-breadcrumbs a").first
    home_link.click()
    expect(page).to_have_url(re.compile(r"index\.html$|/$"))


def test_search_box_exists(page: Page, base_url: str) -> None:
    """The search form is present in the sidebar."""
    page.goto(base_url)
    search_input = page.locator("#rtd-search-form input[name='q']")
    expect(search_input).to_be_visible()


def test_search_returns_results(page: Page, base_url: str) -> None:
    """Searching for 'chunk' produces results on the search page."""
    page.goto(base_url)
    search_input = page.locator("#rtd-search-form input[name='q']")
    search_input.fill("chunk")
    search_input.press("Enter")
    page.wait_for_load_state("networkidle")
    # The search results page should contain at least one result
    results = page.locator("#search-results li")
    expect(results.first).to_be_visible(timeout=10_000)


def test_next_prev_buttons(page: Page, base_url: str) -> None:
    """Next/Previous navigation buttons advance through pages."""
    page.goto(base_url)
    next_link = page.locator("a.btn-neutral", has_text="Next")
    expect(next_link).to_be_visible()
    original_url = page.url
    next_link.click()
    # URL should change after clicking Next
    expect(page).not_to_have_url(original_url)


def test_sidebar_collapse_toggle(page: Page, base_url: str) -> None:
    """The sidebar toggle button collapses and restores the sidebar."""
    page.goto(base_url)
    toggle = page.locator("#sidebar-toggle")
    expect(toggle).to_be_visible()

    # Sidebar should start expanded (no .sidebar-collapsed on html)
    html = page.locator("html")
    expect(html).not_to_have_class(re.compile(r"sidebar-collapsed"))

    # Click to collapse
    toggle.click()
    expect(html).to_have_class(re.compile(r"sidebar-collapsed"))

    # Click to expand again
    toggle.click()
    expect(html).not_to_have_class(re.compile(r"sidebar-collapsed"))
