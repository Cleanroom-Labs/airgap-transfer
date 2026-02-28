"""Needflow diagram interaction tests: clickability, scroll offset, layout balance."""

import pytest
from playwright.sync_api import Page, expect


# ---------------------------------------------------------------------------
# Needflow JS Processing
# ---------------------------------------------------------------------------
class TestNeedflowProcessing:
    """Verify needflow-fix.js replaced native image maps with JS handlers."""

    PAGES_WITH_NEEDFLOWS = [
        pytest.param(
            "/dashboard/usecase-trace.html", 7, id="usecase-trace"
        ),
        pytest.param(
            "/dashboard/req-coverage.html", 9, id="req-coverage"
        ),
    ]

    @pytest.mark.parametrize("path,min_figures", PAGES_WITH_NEEDFLOWS)
    def test_needflow_js_processed_all_figures(
        self, page: Page, base_url: str, path: str, min_figures: int
    ) -> None:
        """All needflow figures have usemap removed (JS click handlers active)."""
        page.goto(f"{base_url}{path}")
        page.wait_for_load_state("networkidle")

        result = page.evaluate("""() => {
            const figs = document.querySelectorAll('figure[id^="needflow-"]');
            let total = figs.length;
            let unprocessed = 0;
            figs.forEach(fig => {
                const img = fig.querySelector('img');
                if (img && img.getAttribute('usemap')) unprocessed++;
            });
            return { total, unprocessed };
        }""")
        assert result["total"] >= min_figures, (
            f"Expected >={min_figures} needflow figures, found {result['total']}"
        )
        assert result["unprocessed"] == 0, (
            f"{result['unprocessed']} of {result['total']} figures still have "
            f"native usemap (needflow-fix.js did not process them)"
        )


# ---------------------------------------------------------------------------
# Needflow Node Click Navigation
# ---------------------------------------------------------------------------
class TestNeedflowNavigation:
    """Clicking a node in a needflow diagram navigates to its spec page."""

    def _click_area_center(
        self, page: Page, figure_index: int,
        nat_x: float, nat_y: float,
    ) -> None:
        """Scroll a needflow figure into view, compute display coords, click."""
        # Compute the absolute display coordinates for the area center.
        coords = page.evaluate("""(args) => {
            const figs = document.querySelectorAll('figure[id^="needflow-"]');
            if (figs.length <= args.idx) return null;
            const img = figs[args.idx].querySelector('img');
            if (!img) return null;
            // Scroll into view so getBoundingClientRect is in-viewport.
            img.scrollIntoView({ block: 'center', behavior: 'instant' });
            const rect = img.getBoundingClientRect();
            const scaleX = rect.width / img.naturalWidth;
            const scaleY = rect.height / img.naturalHeight;
            return {
                x: rect.left + args.natX * scaleX,
                y: rect.top + args.natY * scaleY,
            };
        }""", {"idx": figure_index, "natX": nat_x, "natY": nat_y})
        assert coords is not None, (
            f"Could not compute click coords for figure {figure_index}"
        )
        # Use expect_navigation to wait for the click-triggered navigation.
        with page.expect_navigation(timeout=5000):
            page.mouse.click(coords["x"], coords["y"])

    def test_needflow_node_click_navigates(
        self, page: Page, base_url: str
    ) -> None:
        """Click a known area in the UC-004 diagram (not CSS-scaled)."""
        page.goto(f"{base_url}/dashboard/usecase-trace.html")
        page.wait_for_load_state("networkidle")

        # UC-004 (SBOM Transfer) is figure index 3, natural size 368x340.
        # FR-TRANSFER-048 area: (5,228)-(183,334).  Center: (94, 281).
        self._click_area_center(page, figure_index=3, nat_x=94, nat_y=281)

        assert "FR-TRANSFER-048" in page.url, (
            f"Expected navigation to FR-TRANSFER-048, got: {page.url}"
        )

    def test_wide_diagram_click_navigates(
        self, page: Page, base_url: str
    ) -> None:
        """Click a known area in the UC-001 diagram (CSS-scaled from 1431px)."""
        page.goto(f"{base_url}/dashboard/usecase-trace.html")
        page.wait_for_load_state("networkidle")

        # UC-001 (Large File Transfer) is figure index 0, natural size 1431x302.
        # FR-TRANSFER-001 area: (5,200)-(159,288).  Center: (82, 244).
        self._click_area_center(page, figure_index=0, nat_x=82, nat_y=244)

        assert "FR-TRANSFER-001" in page.url, (
            f"Expected navigation to FR-TRANSFER-001, got: {page.url}"
        )


# ---------------------------------------------------------------------------
# Anchor Scroll Offset
# ---------------------------------------------------------------------------
class TestAnchorScrollOffset:
    """Anchor targets scroll below the fixed navbar."""

    def test_anchor_scroll_clears_navbar(
        self, page: Page, base_url: str
    ) -> None:
        """Navigating to an anchor positions the element below the navbar."""
        page.goto(f"{base_url}/requirements/srs.html#FR-TRANSFER-001")
        page.wait_for_load_state("networkidle")

        top = page.evaluate("""() => {
            const el = document.getElementById('FR-TRANSFER-001');
            if (!el) return -1;
            return el.getBoundingClientRect().top;
        }""")
        # Element should be below the navbar (64px min, 92px with version bar)
        assert top >= 60, (
            f"Anchor target top={top:.0f}px, expected >=60 (below navbar)"
        )


# ---------------------------------------------------------------------------
# Content Layout Balance
# ---------------------------------------------------------------------------
class TestContentBalance:
    """Document content is centered within the content area."""

    VIEWPORTS = [
        pytest.param(1440, id="1440px"),
        pytest.param(1920, id="1920px"),
    ]

    @pytest.mark.parametrize("width", VIEWPORTS)
    def test_content_centered_with_sidebar(
        self, page: Page, base_url: str, width: int,
    ) -> None:
        """With sidebar expanded, .document has equal whitespace on each side.

        At 1920px the .wy-nav-content max-width (1200px) kicks in, so this
        also verifies that .wy-nav-content itself is centered within the
        content-wrap area.
        """
        page.set_viewport_size({"width": width, "height": 900})
        page.goto(base_url)
        page.wait_for_load_state("domcontentloaded")

        gaps = page.evaluate("""() => {
            const doc = document.querySelector('.document');
            const sidebar = document.querySelector('.wy-nav-side');
            if (!doc || !sidebar) return null;
            const docRect = doc.getBoundingClientRect();
            const sidebarRight = sidebar.getBoundingClientRect().right;
            return {
                leftGap: docRect.left - sidebarRight,
                rightGap: window.innerWidth - docRect.right,
            };
        }""")
        assert gaps is not None, ".document or .wy-nav-side not found"
        diff = abs(gaps["leftGap"] - gaps["rightGap"])
        assert diff <= 30, (
            f"Content not centered at {width}px: "
            f"left gap={gaps['leftGap']:.0f}px, "
            f"right gap={gaps['rightGap']:.0f}px (diff={diff:.0f}px)"
        )
