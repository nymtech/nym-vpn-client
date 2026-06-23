import Testing
import AccountPrefetchGates

struct AuthCarouselLayoutPolicyTests {
    @Test func pinnedDrawerHeightUsesRootMinimumWhenTaller() {
        #expect(AuthCarouselLayoutPolicy.pinnedDrawerHeight(rootMinHeight: 320) == 320)
    }

    @Test func pinnedDrawerHeightUsesCarouselMinimumWhenRootIsShorter() {
        #expect(
            AuthCarouselLayoutPolicy.pinnedDrawerHeight(rootMinHeight: 200)
                == AuthCarouselLayoutPolicy.minimumCarouselDrawerHeight
        )
    }
}
