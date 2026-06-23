import Testing
import AccountPrefetchGates

struct AuthCarouselLayoutPolicyTests {
    @Test func pinnedDrawerHeightUsesRootMinimumWhenTaller() {
        let tallerThanCarousel = AuthCarouselLayoutPolicy.minimumCarouselDrawerHeight + 20
        #expect(AuthCarouselLayoutPolicy.pinnedDrawerHeight(rootMinHeight: tallerThanCarousel) == tallerThanCarousel)
    }

    @Test func minimumCarouselDrawerHeightIncludesStackSpacingAndTitleBlock() {
        #expect(AuthCarouselLayoutPolicy.minimumCarouselDrawerHeight == 388)
    }

    @Test func pinnedDrawerHeightUsesCarouselMinimumWhenRootIsShorter() {
        #expect(
            AuthCarouselLayoutPolicy.pinnedDrawerHeight(rootMinHeight: 200)
                == AuthCarouselLayoutPolicy.minimumCarouselDrawerHeight
        )
    }
}
