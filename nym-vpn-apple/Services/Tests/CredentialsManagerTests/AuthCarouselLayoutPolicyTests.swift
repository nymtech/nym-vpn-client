import Testing
import AccountPrefetchGates

struct AuthCarouselLayoutPolicyTests {
    @Test func pinnedDrawerHeightUsesRootMinimum() {
        #expect(AuthCarouselLayoutPolicy.pinnedDrawerHeight(rootMinHeight: 320) == 320)
        #expect(AuthCarouselLayoutPolicy.pinnedDrawerHeight(rootMinHeight: 0) == 0)
    }
}
