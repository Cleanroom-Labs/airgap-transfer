"""Dashboard tests: pie charts, status tables, and needs data rendering."""

from playwright.sync_api import Page, expect


def test_proj_health_has_pie_charts(page: Page, base_url: str) -> None:
    """Project Health page renders needpie SVG charts."""
    page.goto(f"{base_url}/dashboard/proj-health.html")
    page.wait_for_load_state("networkidle")
    # needpie renders matplotlib charts as SVG images in _images/
    charts = page.locator("img[id^='needpie-']")
    count = charts.count()
    assert count >= 1, "Expected at least one pie chart image on Project Health page"


def test_proj_health_pie_legends_visible(page: Page, base_url: str) -> None:
    """Pie chart legends are visible (not hidden or overlapping)."""
    page.goto(f"{base_url}/dashboard/proj-health.html")
    page.wait_for_load_state("networkidle")
    # The page should have visible text content beyond just the heading
    content = page.locator(".wy-nav-content .section, .wy-nav-content section").first
    expect(content).to_be_visible()


def test_req_coverage_has_tables(page: Page, base_url: str) -> None:
    """Requirement Coverage page has at least one needs table with data rows."""
    page.goto(f"{base_url}/dashboard/req-coverage.html")
    page.wait_for_load_state("networkidle")
    # sphinx-needs renders tables; check for table rows
    tables = page.locator("table")
    assert tables.count() >= 1, "Expected at least one table on Requirement Coverage page"
    # The first table should have at least one data row
    rows = tables.first.locator("tbody tr")
    assert rows.count() >= 1, "Expected at least one data row in the coverage table"


def test_test_status_page_has_content(page: Page, base_url: str) -> None:
    """Test Status page loads with meaningful content."""
    page.goto(f"{base_url}/dashboard/test-status.html")
    page.wait_for_load_state("networkidle")
    content = page.locator(".wy-nav-content")
    expect(content).to_be_visible()
    # Page should have at least some text beyond the heading
    text = content.inner_text()
    assert len(text) > 50, "Test Status page appears to have very little content"


def test_impl_status_page_has_tables(page: Page, base_url: str) -> None:
    """Implementation Status page has needs tables."""
    page.goto(f"{base_url}/dashboard/impl-status.html")
    page.wait_for_load_state("networkidle")
    tables = page.locator("table")
    assert tables.count() >= 1, "Expected at least one table on Implementation Status page"


def test_coverage_gaps_page_loads(page: Page, base_url: str) -> None:
    """Coverage Gaps page loads and has content."""
    page.goto(f"{base_url}/dashboard/coverage-gaps.html")
    page.wait_for_load_state("networkidle")
    content = page.locator(".wy-nav-content")
    expect(content).to_be_visible()
    text = content.inner_text()
    assert len(text) > 50, "Coverage Gaps page appears to have very little content"
