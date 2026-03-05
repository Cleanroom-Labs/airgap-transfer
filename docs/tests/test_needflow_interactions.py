"""Content-specific needflow tests: click navigation with known coordinates.

Theme-generic needflow tests (JS processing, cursor behavior, layout balance,
anchor scroll offset) are in the common submodule's test suite.  This file
tests only click navigation using AirGap-specific diagram coordinates.
"""

import re

from playwright.sync_api import Page


class TestNeedflowNavigation:
    """Clicking a node in a needflow diagram navigates to its spec page."""

    def _find_area_center_in_source(
        self, page: Page, href_fragment: str
    ) -> tuple[float, float]:
        """Parse source HTML and return the center of the target <area>."""
        response = page.context.request.get(page.url)
        assert response.ok, f"Failed to fetch page source: {page.url}"
        html = response.text()

        for area_tag in re.findall(r"<area\\b[^>]*>", html, flags=re.IGNORECASE):
            href_match = re.search(
                r'href="([^"]+)"', area_tag, flags=re.IGNORECASE
            )
            coords_match = re.search(
                r'coords="([0-9,]+)"', area_tag, flags=re.IGNORECASE
            )
            if not href_match or not coords_match:
                continue
            if href_fragment not in href_match.group(1):
                continue
            x1, y1, x2, y2 = (float(v) for v in coords_match.group(1).split(","))
            return ((x1 + x2) / 2, (y1 + y2) / 2)

        raise AssertionError(
            f"Could not find <area> for href fragment {href_fragment} in {page.url}"
        )

    def _click_area_by_href(
        self, page: Page, figure_index: int, href_fragment: str
    ) -> None:
        """Click the center of a needflow area selected by href fragment."""
        nat_x, nat_y = self._find_area_center_in_source(page, href_fragment)
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
            "Could not compute click coords for "
            f"figure {figure_index}, href fragment {href_fragment}"
        )
        with page.expect_navigation(timeout=5000):
            page.mouse.click(coords["x"], coords["y"])

    def test_needflow_node_click_navigates(
        self, page: Page, base_url: str
    ) -> None:
        """Click FR-TRANSFER-048 in the UC-004 diagram."""
        page.goto(f"{base_url}/use-cases/future.html")
        page.wait_for_load_state("networkidle")

        # UC-004 (SBOM Transfer) is the only needflow on this page (index 0).
        self._click_area_by_href(
            page, figure_index=0, href_fragment="FR-TRANSFER-048"
        )

        assert "FR-TRANSFER-048" in page.url, (
            f"Expected navigation to FR-TRANSFER-048, got: {page.url}"
        )

    def test_wide_diagram_click_navigates(
        self, page: Page, base_url: str
    ) -> None:
        """Click FR-TRANSFER-001 in the UC-001 diagram."""
        page.goto(f"{base_url}/use-cases/use-case-large-file.html")
        page.wait_for_load_state("networkidle")

        # UC-001 (Large File Transfer) is the only needflow on this page (index 0).
        self._click_area_by_href(
            page, figure_index=0, href_fragment="FR-TRANSFER-001"
        )

        assert "FR-TRANSFER-001" in page.url, (
            f"Expected navigation to FR-TRANSFER-001, got: {page.url}"
        )
