import SwiftUI
import Theme

#if os(iOS)
import UIKit
private typealias PlatformFont = UIFont
private typealias PlatformColor = UIColor
#elseif os(macOS)
import AppKit
private typealias PlatformFont = NSFont
private typealias PlatformColor = NSColor
#endif

private enum LogTextStyle {
    static let fontSize: CGFloat = 11

    static var font: PlatformFont {
        PlatformFont.monospacedSystemFont(ofSize: fontSize, weight: .regular)
    }

    static var defaultTextColor: PlatformColor {
#if os(iOS)
        return .label
#elseif os(macOS)
        return .labelColor
#endif
    }

    static var timestampColor: PlatformColor {
        PlatformColor(Color.Nym.info)
    }

    /// Matches both formats produced in this app:
    ///   - Swift `Date()` description: `2026-04-10 07:54:59 +0000`
    ///   - Rust `tracing` RFC3339: `2026-04-29T13:45:23.123456Z` / with `±HH:MM` offset
    static let timestampRegex: NSRegularExpression? = try? NSRegularExpression(
        pattern: #"^\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}(?:[.,]\d+)?(?:\s?[+-]\d{2}:?\d{2}|Z)?"#,
        options: [.anchorsMatchLines]
    )

    static func attributed(from text: String) -> NSAttributedString {
        let baseAttrs: [NSAttributedString.Key: Any] = [
            .font: font,
            .foregroundColor: defaultTextColor
        ]
        let result = NSMutableAttributedString(string: text, attributes: baseAttrs)
        let nsText = text as NSString
        let fullRange = NSRange(location: 0, length: nsText.length)
        timestampRegex?.enumerateMatches(in: text, options: [], range: fullRange) { match, _, _ in
            guard let range = match?.range else {
                return
            }
            result.addAttribute(.foregroundColor, value: timestampColor, range: range)
        }
        return result
    }
}

#if os(iOS)
struct LogTextView: UIViewRepresentable {
    let text: String
    let scrollIntent: LogScrollIntent

    func makeCoordinator() -> Coordinator {
        Coordinator()
    }

    func makeUIView(context: Context) -> UITextView {
        let view = UITextView()
        view.isEditable = false
        view.isSelectable = true
        view.backgroundColor = .clear
        view.textContainerInset = UIEdgeInsets(top: 8, left: 8, bottom: 8, right: 8)
        view.textContainer.lineBreakMode = .byCharWrapping
        view.alwaysBounceVertical = true
        view.dataDetectorTypes = []
        view.adjustsFontForContentSizeCategory = false
        view.layoutManager.allowsNonContiguousLayout = true
        return view
    }

    func updateUIView(_ uiView: UITextView, context: Context) {
        let coordinator = context.coordinator
        guard coordinator.lastAppliedText != text else {
            return
        }
        let previousHeight = uiView.contentSize.height
        let previousOffsetY = uiView.contentOffset.y

        uiView.attributedText = LogTextStyle.attributed(from: text)
        coordinator.lastAppliedText = text

        DispatchQueue.main.async {
            switch scrollIntent {
            case .bottom:
                let length = uiView.attributedText.length
                guard length > 0 else { return }
                uiView.scrollRangeToVisible(NSRange(location: length - 1, length: 1))
            case .preserve:
                let newHeight = uiView.contentSize.height
                let delta = newHeight - previousHeight
                let target = max(0, previousOffsetY + delta)
                uiView.setContentOffset(CGPoint(x: 0, y: target), animated: false)
            case .idle:
                break
            }
        }
    }

    final class Coordinator {
        var lastAppliedText: String?
    }
}
#elseif os(macOS)
struct LogTextView: NSViewRepresentable {
    let text: String
    let scrollIntent: LogScrollIntent

    func makeCoordinator() -> Coordinator {
        Coordinator()
    }

    func makeNSView(context: Context) -> NSScrollView {
        let scroll = NSTextView.scrollableTextView()
        scroll.drawsBackground = false
        scroll.hasVerticalScroller = true
        scroll.autohidesScrollers = true

        guard let textView = scroll.documentView as? NSTextView else {
            return scroll
        }
        textView.isEditable = false
        textView.isSelectable = true
        textView.drawsBackground = false
        textView.textContainerInset = NSSize(width: 8, height: 8)
        textView.isAutomaticQuoteSubstitutionEnabled = false
        textView.isAutomaticDashSubstitutionEnabled = false
        textView.isAutomaticTextReplacementEnabled = false
        textView.isRichText = false
        textView.allowsUndo = false
        textView.layoutManager?.allowsNonContiguousLayout = true
        return scroll
    }

    func updateNSView(_ nsView: NSScrollView, context: Context) {
        guard let textView = nsView.documentView as? NSTextView else {
            return
        }
        let coordinator = context.coordinator
        guard coordinator.lastAppliedText != text else {
            return
        }
        let clipView = nsView.contentView
        let previousHeight = textView.frame.height
        let previousOriginY = clipView.bounds.origin.y

        textView.textStorage?.setAttributedString(LogTextStyle.attributed(from: text))
        coordinator.lastAppliedText = text

        DispatchQueue.main.async {
            if let container = textView.textContainer {
                textView.layoutManager?.ensureLayout(for: container)
            }
            switch scrollIntent {
            case .bottom:
                textView.scrollToEndOfDocument(nil)
            case .preserve:
                let newHeight = textView.frame.height
                let delta = newHeight - previousHeight
                let target = max(0, previousOriginY + delta)
                clipView.scroll(to: NSPoint(x: 0, y: target))
                nsView.reflectScrolledClipView(clipView)
            case .idle:
                break
            }
        }
    }

    final class Coordinator {
        var lastAppliedText: String?
    }
}
#endif
