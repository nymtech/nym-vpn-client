import Testing
@testable import Home

struct OneClickCaretSlotTests {
    @Test func collapsedCaretStacksOnExitChoice() {
        #expect(OneClickDisplayMode.powerUser.caretSlot == .exitInfoStack)
    }

    @Test func expandedCaretSitsOnExitLabel() {
        #expect(OneClickDisplayMode.nerd.caretSlot == .exitLabel)
    }
}
