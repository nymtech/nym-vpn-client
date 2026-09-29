import Testing
@testable import Home

struct OneClickCaretSlotTests {
    @Test func nerdCaretSitsBetweenHops() {
        #expect(OneClickDisplayMode.nerd.caretSlot == .betweenHops)
    }

    @Test func powerUserCaretStaysOnExitRow() {
        #expect(OneClickDisplayMode.powerUser.caretSlot == .exitRow)
    }
}
