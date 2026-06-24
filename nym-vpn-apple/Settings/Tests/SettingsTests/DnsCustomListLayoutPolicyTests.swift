import Testing
@testable import Settings

struct DnsCustomListLayoutPolicyTests {
    @Test func emptyCustomDnsSkipsReorderableList() {
        #expect(DnsCustomListLayoutPolicy.shouldRenderReorderableList(entryCount: 0) == false)
    }

    @Test func nonEmptyCustomDnsShowsReorderableList() {
        #expect(DnsCustomListLayoutPolicy.shouldRenderReorderableList(entryCount: 1) == true)
        #expect(DnsCustomListLayoutPolicy.shouldRenderReorderableList(entryCount: 5) == true)
    }
}
