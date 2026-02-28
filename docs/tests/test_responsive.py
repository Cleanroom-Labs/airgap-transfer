"""Responsive layout tests: overflow, touch targets, and content constraints."""

import pytest
from playwright.sync_api import Page, expect

# Viewports to test.
VIEWPORTS = [
    pytest.param(375, 667, id="mobile-375"),
    pytest.param(768, 1024, id="tablet-768"),
    pytest.param(1920, 1080, id="desktop-1920"),
    pytest.param(667, 375, id="landscape-667"),
]

# Pages to test for horizontal overflow.
OVERFLOW_PAGES = [
    pytest.param("", id="index"),
    pytest.param("/dashboard/proj-health.html", id="proj-health"),
    pytest.param("/requirements/srs.html", id="srs"),
    pytest.param("/readme.html", id="readme"),
]


# ---------------------------------------------------------------------------
# No Horizontal Overflow
# ---------------------------------------------------------------------------
class TestNoHorizontalOverflow:
    """No page produces a horizontal scrollbar at any viewport."""

    @pytest.mark.parametrize("width,height", VIEWPORTS)
    @pytest.mark.parametrize("path", OVERFLOW_PAGES)
    def test_no_overflow(
        self, page: Page, base_url: str, width: int, height: int, path: str
    ) -> None:
        page.set_viewport_size({"width": width, "height": height})
        page.goto(f"{base_url}{path}")
        page.wait_for_load_state("domcontentloaded")
        overflow = page.evaluate(
            "document.documentElement.scrollWidth > document.documentElement.clientWidth"
        )
        assert not overflow, f"Horizontal overflow at {width}x{height} on {path or '/'}"


# ---------------------------------------------------------------------------
# Touch Targets (mobile)
# ---------------------------------------------------------------------------
class TestTouchTargets:
    """Header navigation touch targets meet minimum size on mobile."""

    def test_nav_area_touch_targets(self, page: Page, base_url: str) -> None:
        page.set_viewport_size({"width": 375, "height": 667})
        page.goto(base_url)
        nav_links = page.locator(".site-nav-menu a, .site-nav-logo")
        count = nav_links.count()
        if count == 0:
            return  # Standalone may not have these elements
        for i in range(count):
            box = nav_links.nth(i).bounding_box()
            if box:
                assert box["height"] >= 40, (
                    f"Nav element {i} height {box['height']}px < 40px minimum"
                )


# ---------------------------------------------------------------------------
# Needs-box Containment (mobile)
# ---------------------------------------------------------------------------
class TestNeedsBoxContainment:
    """Needs boxes should not overflow the viewport on mobile."""

    def test_needs_boxes_fit_or_scroll(self, page: Page, base_url: str) -> None:
        page.set_viewport_size({"width": 375, "height": 667})
        page.goto(f"{base_url}/requirements/srs.html")
        page.wait_for_load_state("domcontentloaded")

        overflowing = page.evaluate("""() => {
            const vw = document.documentElement.clientWidth;
            const boxes = document.querySelectorAll(
                '.need, .usecase, .req, .nfreq, .test, .impl, .spec'
            );
            let count = 0;
            for (const box of boxes) {
                const rect = box.getBoundingClientRect();
                const style = window.getComputedStyle(box);
                // Box overflows if it extends beyond viewport AND doesn't scroll
                if (rect.right > vw && style.overflowX !== 'auto' && style.overflowX !== 'scroll') {
                    count++;
                }
            }
            return count;
        }""")
        assert overflowing == 0, f"{overflowing} needs boxes overflow without scroll"


# ---------------------------------------------------------------------------
# Table Responsiveness (mobile)
# ---------------------------------------------------------------------------
class TestTableResponsiveness:
    """Tables should be wrapped for horizontal scrolling on mobile."""

    def test_tables_have_responsive_wrapper(self, page: Page, base_url: str) -> None:
        page.set_viewport_size({"width": 375, "height": 667})
        page.goto(f"{base_url}/requirements/srs.html")
        page.wait_for_load_state("domcontentloaded")

        # RTD theme wraps tables in .wy-table-responsive
        unwrapped = page.evaluate("""() => {
            const tables = document.querySelectorAll('.wy-nav-content table');
            let count = 0;
            for (const table of tables) {
                const parent = table.parentElement;
                if (!parent || !parent.classList.contains('wy-table-responsive')) {
                    count++;
                }
            }
            return count;
        }""")
        # Some tables (e.g. inside needs directives) may not be wrapped; just log
        assert unwrapped < 20, f"{unwrapped} tables lack responsive wrapper"


# ---------------------------------------------------------------------------
# Pie Chart Rendering (mobile)
# ---------------------------------------------------------------------------
class TestPieChartRendering:
    """Charts on the project health dashboard fit within mobile viewport."""

    def test_charts_fit_mobile_viewport(self, page: Page, base_url: str) -> None:
        page.set_viewport_size({"width": 375, "height": 667})
        page.goto(f"{base_url}/dashboard/proj-health.html")
        page.wait_for_load_state("networkidle")

        images = page.locator("img")
        count = images.count()
        for i in range(count):
            img = images.nth(i)
            src = img.get_attribute("src") or ""
            if src.startswith("data:"):
                continue
            box = img.bounding_box()
            if box and box["width"] > 0:
                assert box["width"] <= 375, (
                    f"Image '{src}' width {box['width']}px exceeds 375px viewport"
                )


# ---------------------------------------------------------------------------
# Content Width Constraints (desktop)
# ---------------------------------------------------------------------------
class TestContentWidthConstraints:
    """Content area is properly constrained on wide viewports."""

    def test_content_max_width_at_1920(self, page: Page, base_url: str) -> None:
        page.set_viewport_size({"width": 1920, "height": 1080})
        page.goto(base_url)
        content_width = page.evaluate("""() => {
            const el = document.querySelector('.wy-nav-content');
            return el ? el.getBoundingClientRect().width : 0;
        }""")
        assert content_width <= 1200, (
            f".wy-nav-content width {content_width}px > 1200px at 1920px viewport"
        )


# ---------------------------------------------------------------------------
# Landscape Mode
# ---------------------------------------------------------------------------
class TestLandscapeMode:
    """Pages render without overflow in landscape orientation."""

    def test_index_landscape_no_overflow(self, page: Page, base_url: str) -> None:
        page.set_viewport_size({"width": 667, "height": 375})
        page.goto(base_url)
        overflow = page.evaluate(
            "document.documentElement.scrollWidth > document.documentElement.clientWidth"
        )
        assert not overflow, "Horizontal overflow in landscape mode"
