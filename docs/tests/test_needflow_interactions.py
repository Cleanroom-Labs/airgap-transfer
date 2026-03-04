"""Content-specific needflow tests: click navigation with known coordinates.

Theme-generic needflow tests (JS processing, cursor behavior, layout balance,
anchor scroll offset) are in the common submodule's test suite.  This file
tests only click navigation using AirGap-specific diagram coordinates.
"""

from playwright.sync_api import Page


class TestNeedflowNavigation:
    """Clicking a node in a needflow diagram navigates to its spec page."""

    def _click_area_center(
        self, page: Page, figure_index: int,
        nat_x: float, nat_y: float,
    ) -> None:
        """Scroll a needflow figure into view, compute display coords, click."""
        coords = page.evaluate("""(args) => {
            const figs = document.querySelectorAll('figure[id^="needflow-"]');
            if (figs.length <= args.idx) return null;
            const img = figs[args.idx].querySelector('img');
            if (!img) return null;
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
        with page.expect_navigation(timeout=5000):
            page.mouse.click(coords["x"], coords["y"])

    def test_needflow_node_click_navigates(
        self, page: Page, base_url: str
    ) -> None:
        """Click a known area in the UC-004 diagram (not CSS-scaled)."""
        page.goto(f"{base_url}/use-cases/future.html")
        page.wait_for_load_state("networkidle")

        # UC-004 (SBOM Transfer) is the only needflow on this page (index 0).
        # Natural size 368x340.
        # FR-TRANSFER-048 area: (186,228)-(363,334).  Center: (274, 281).
        self._click_area_center(page, figure_index=0, nat_x=274, nat_y=281)

        assert "FR-TRANSFER-048" in page.url, (
            f"Expected navigation to FR-TRANSFER-048, got: {page.url}"
        )

    def test_wide_diagram_click_navigates(
        self, page: Page, base_url: str
    ) -> None:
        """Click a known area in the UC-001 diagram (CSS-scaled from 2216px)."""
        page.goto(f"{base_url}/use-cases/use-case-large-file.html")
        page.wait_for_load_state("networkidle")

        # UC-001 (Large File Transfer) is the only needflow on this page (index 0).
        # Natural size 1896x506.
        # FR-TRANSFER-001 area: (1372,200)-(1525,288).  Center: (1448, 244).
        self._click_area_center(page, figure_index=0, nat_x=1448, nat_y=244)

        assert "FR-TRANSFER-001" in page.url, (
            f"Expected navigation to FR-TRANSFER-001, got: {page.url}"
        )
