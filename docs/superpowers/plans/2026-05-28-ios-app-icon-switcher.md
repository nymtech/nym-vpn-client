# iOS App Icon Switcher Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Settings → Appearance → App icon picker that lets users switch among Default, Dark, Calculator, and Notes icons using `UIApplication.shared.setAlternateIconName`.

**Architecture:** An `AppIcon` enum lives in the `AppSettings` target (alongside `AppSetting`). A `AppIconViewModel` (in the `Settings` target, mirroring `DisplayThemeViewModel`) drives an `AppIconView` that uses existing `SettingButton`/`SettingsListItem` components. The route is wired through `SettingsFlowCoordinator` exactly as `displayTheme`. Three placeholder 1024×1024 solid-color PNGs act as alternate icon assets until a designer replaces them.

**Tech Stack:** Swift/SwiftUI, UIKit (`UIApplication.setAlternateIconName`), Xcode asset catalog alternates (`ASSETCATALOG_COMPILER_ALTERNATE_APPICON_NAMES`), String Catalog (`.xcstrings`).

---

## File Map

### New files
| File | Responsibility |
|---|---|
| `Services/Sources/Services/AppSettings/Settings/AppIcon.swift` | `AppIcon` enum: cases, `alternateName`, `previewAssetName`, `localizedTitleKey` |
| `Settings/Sources/Settings/Theme/AppIcon/AppIconViewModel.swift` | Reads current icon from UIKit, handles change request + async confirm |
| `Settings/Sources/Settings/Theme/AppIcon/AppIconView.swift` | SwiftUI grid of icon cards using `SettingButton` style; confirmation alert |
| `NymVPN/Assets.xcassets/AppIcon-Dark.appiconset/Contents.json` | Asset catalog descriptor for Dark alternate |
| `NymVPN/Assets.xcassets/AppIcon-Dark.appiconset/AppIcon-Dark.png` | 1024×1024 placeholder (#0F1A2A) |
| `NymVPN/Assets.xcassets/AppIcon-Calculator.appiconset/Contents.json` | Asset catalog descriptor for Calculator alternate |
| `NymVPN/Assets.xcassets/AppIcon-Calculator.appiconset/AppIcon-Calculator.png` | 1024×1024 placeholder (#1A1F2E) |
| `NymVPN/Assets.xcassets/AppIcon-Notes.appiconset/Contents.json` | Asset catalog descriptor for Notes alternate |
| `NymVPN/Assets.xcassets/AppIcon-Notes.appiconset/AppIcon-Notes.png` | 1024×1024 placeholder (#F5C84B) |

### Modified files
| File | Change |
|---|---|
| `Services/Sources/Services/AppSettings/AppSettings.swift` | Add `case appIcon` to `AppSettingKey`; add `@AppStorage` `appIconRawValue` + computed `appIcon` on iOS |
| `Settings/Sources/Settings/SettingLink.swift` | Add `case appIcon` |
| `Settings/Sources/Settings/Theme/AppearanceView.swift` | Add `appIcon()` row in `body`, add `#if os(iOS)` guard since alternates are iOS-only |
| `Settings/Sources/Settings/Theme/AppearanceView+Actions.swift` | Add `navigateToAppIcon()` |
| `Settings/Sources/Settings/SettingsFlowCoordinator.swift` | Add `case .appIcon` branch + `appIconDestination()` helper |
| `NymVPN.xcodeproj/project.pbxproj` | Add `ASSETCATALOG_COMPILER_ALTERNATE_APPICON_NAMES` to the iOS NymVPN target Debug + Release build settings (IDs `D99A14852B357E9A00F2728B` and `D99A14862B357E9A00F2728B`) |
| `NymVPN/Resources/Localizable.xcstrings` | Add 9 new English-only keys under `settings.appIcon.*` |

---

## Task 1: Create the `AppIcon` enum

**Files:**
- Create: `nym-vpn-apple/Services/Sources/Services/AppSettings/Settings/AppIcon.swift`

- [ ] **Step 1: Create the file**

```swift
import Foundation

public enum AppIcon: String, CaseIterable, Identifiable, Sendable {
    case `default`
    case dark
    case calculator
    case notes

    public var id: String { rawValue }

    /// The name passed to `UIApplication.shared.setAlternateIconName`.
    /// `nil` resets to the primary AppIcon.
    public var alternateName: String? {
        switch self {
        case .default:    return nil
        case .dark:       return "AppIcon-Dark"
        case .calculator: return "AppIcon-Calculator"
        case .notes:      return "AppIcon-Notes"
        }
    }

    /// Asset-catalog name used for the preview image in the picker.
    public var previewAssetName: String {
        switch self {
        case .default:    return "AppIcon"
        case .dark:       return "AppIcon-Dark"
        case .calculator: return "AppIcon-Calculator"
        case .notes:      return "AppIcon-Notes"
        }
    }

    public var localizedTitleKey: String {
        switch self {
        case .default:    return "settings.appIcon.default"
        case .dark:       return "settings.appIcon.dark"
        case .calculator: return "settings.appIcon.calculator"
        case .notes:      return "settings.appIcon.notes"
        }
    }
}
```

- [ ] **Step 2: Verify the file compiles in isolation**

```bash
cd /home/alexis/Apps/nym/nym-vpn-client/nym-vpn-apple/Services && swift build --target AppSettings 2>&1 | tail -20
```
Expected: build succeeds (no errors about `AppIcon`). If `swift build` is unavailable (non-macOS CI), note the limitation and continue.

- [ ] **Step 3: Commit**

```bash
cd /home/alexis/Apps/nym/nym-vpn-client
git add nym-vpn-apple/Services/Sources/Services/AppSettings/Settings/AppIcon.swift
git commit -m "feat(ios): add AppIcon enum with four icon cases"
```

---

## Task 2: Extend `AppSettings` with `appIcon` storage

**Files:**
- Modify: `nym-vpn-apple/Services/Sources/Services/AppSettings/AppSettings.swift` (lines 226–258 — the `AppSettingKey` enum)

The `@AppStorage` for `appIconRawValue` is iOS-only because `UIApplication.setAlternateIconName` doesn't exist on macOS. The `appIcon` computed property is also iOS-only for the same reason.

- [ ] **Step 1: Add `case appIcon` to `AppSettingKey`**

In `AppSettings.swift`, find the `AppSettingKey` enum (currently ends with `case oneClickDisplayMode`). Add the new case after `oneClickDisplayMode`:

```swift
    case oneClickDisplayMode
    case appIcon
```

- [ ] **Step 2: Add `appIconRawValue` stored property and `appIcon` computed property inside `AppSettings`**

After the `@AppStorage(AppSettingKey.oneClickDisplayMode.rawValue)` property (line ~141), add inside the `#if os(iOS)` block. The existing iOS block ends at the closing `#else` for `isStatisticsEnabled`. Since iOS-only properties are scattered, the cleanest approach is to add after the `oneClickDisplayModeRaw` property with an `#if os(iOS)` guard:

Locate this line (currently around line 141):
```swift
    @AppStorage(AppSettingKey.oneClickDisplayMode.rawValue)
    public var oneClickDisplayModeRaw: String = "powerUser"
```

Add immediately after it:

```swift
#if os(iOS)
    @AppStorage(AppSettingKey.appIcon.rawValue)
    public var appIconRawValue: String = AppIcon.default.rawValue

    public var appIcon: AppIcon {
        get { AppIcon(rawValue: appIconRawValue) ?? .default }
        set { appIconRawValue = newValue.rawValue }
    }
#endif
```

- [ ] **Step 3: Build the AppSettings target**

```bash
cd /home/alexis/Apps/nym/nym-vpn-client/nym-vpn-apple/Services && swift build --target AppSettings 2>&1 | tail -20
```
Expected: succeeds.

- [ ] **Step 4: Commit**

```bash
cd /home/alexis/Apps/nym/nym-vpn-client
git add nym-vpn-apple/Services/Sources/Services/AppSettings/AppSettings.swift
git commit -m "feat(ios): add appIcon AppStorage key to AppSettings"
```

---

## Task 3: Add `case appIcon` to `SettingLink`

**Files:**
- Modify: `nym-vpn-apple/Settings/Sources/Settings/SettingLink.swift`

- [ ] **Step 1: Add the case**

The `SettingLink` enum is iOS+macOS but alternate icons are iOS-only, so wrap the case:

Find:
```swift
    case displayTheme
```

Add after it:
```swift
#if os(iOS)
    case appIcon
#endif
```

- [ ] **Step 2: Commit**

```bash
cd /home/alexis/Apps/nym/nym-vpn-client
git add nym-vpn-apple/Settings/Sources/Settings/SettingLink.swift
git commit -m "feat(ios): add appIcon case to SettingLink"
```

---

## Task 4: Wire navigation in `AppearanceView` and `AppearanceView+Actions`

**Files:**
- Modify: `nym-vpn-apple/Settings/Sources/Settings/Theme/AppearanceView.swift`
- Modify: `nym-vpn-apple/Settings/Sources/Settings/Theme/AppearanceView+Actions.swift`

- [ ] **Step 1: Add the `appIcon()` helper in `AppearanceView.swift`**

Find the closing brace of the existing `theme()` function (~line 82):

```swift
    func theme() -> some View {
        SettingsListItem(
            viewModel: SettingsListItemViewModel(
                accessory: .arrow,
                title: "settings.displayTheme".localizedString,
                imageName: "displayTheme",
                position: SettingsListItemPosition(isFirst: true, isLast: true),
                action: {
                    navigateToDisplayTheme()
                }
            )
        )
    }
```

Add a new `appIcon()` helper immediately after the closing `}` of `theme()`, inside the same `extension AppearanceView` block, guarded with `#if os(iOS)`:

```swift
#if os(iOS)
    func appIcon() -> some View {
        SettingsListItem(
            viewModel: SettingsListItemViewModel(
                accessory: .arrow,
                title: "settings.appIcon.title".localizedString,
                systemImageName: "square.grid.2x2",
                position: SettingsListItemPosition(isFirst: true, isLast: true),
                action: {
                    navigateToAppIcon()
                }
            )
        )
    }
#endif
```

- [ ] **Step 2: Render the `appIcon()` row in `body`**

In the `body`, find the `theme()` call and the surrounding structure:

```swift
                theme()
                    .frame(maxWidth: MagicNumbers.maxWidth)
```

Add the `appIcon()` row immediately after, guarded with `#if os(iOS)`:

```swift
#if os(iOS)
                Spacer()
                    .frame(height: 24)
                appIcon()
                    .frame(maxWidth: MagicNumbers.maxWidth)
#endif
```

- [ ] **Step 3: Add `navigateToAppIcon()` in `AppearanceView+Actions.swift`**

Find the `navigateToDisplayTheme()` function:
```swift
    func navigateToDisplayTheme() {
        path.append(SettingLink.displayTheme)
    }
```

Add after it (inside `#if os(iOS)` since `SettingLink.appIcon` is iOS-only):

```swift
#if os(iOS)
    func navigateToAppIcon() {
        path.append(SettingLink.appIcon)
    }
#endif
```

- [ ] **Step 4: Check `SettingsListItemViewModel` accepts `systemImageName`**

Run:
```bash
grep -n "systemImageName\|imageName" /home/alexis/Apps/nym/nym-vpn-client/nym-vpn-apple/UIComponents/Sources/UIComponents/25/SettingsList/SettingsListItemViewModel.swift | head -20
```

If `systemImageName` is supported (the macOS `appMode()` in `AppearanceView.swift` uses it), proceed. If the `SettingsListItemViewModel` doesn't have it but has only `imageName`, substitute `imageName: "square.grid.2x2"` (the SF Symbol name as a string, which may not render perfectly but won't crash). Use whichever parameter the ViewModel actually accepts.

- [ ] **Step 5: Commit**

```bash
cd /home/alexis/Apps/nym/nym-vpn-client
git add nym-vpn-apple/Settings/Sources/Settings/Theme/AppearanceView.swift nym-vpn-apple/Settings/Sources/Settings/Theme/AppearanceView+Actions.swift
git commit -m "feat(ios): add App icon row to AppearanceView"
```

---

## Task 5: Create `AppIconViewModel`

**Files:**
- Create: `nym-vpn-apple/Settings/Sources/Settings/Theme/AppIcon/AppIconViewModel.swift`

- [ ] **Step 1: Create the directory and file**

```bash
mkdir -p /home/alexis/Apps/nym/nym-vpn-client/nym-vpn-apple/Settings/Sources/Settings/Theme/AppIcon
```

Then create the file:

```swift
#if os(iOS)
import Foundation
import SwiftUI
import UIKit
import AppSettings
import Theme

@MainActor
public final class AppIconViewModel: ObservableObject {
    @Published public var selectedIcon: AppIcon
    @Published public var pendingIcon: AppIcon?
    @Published public var errorMessage: String?

    let title = "settings.appIcon.title".localizedString

    @Binding private var path: NavigationPath
    private let appSettings: AppSettings

    public init(path: Binding<NavigationPath>, appSettings: AppSettings) {
        _path = path
        self.appSettings = appSettings
        let currentAlternateName = UIApplication.shared.alternateIconName
        self.selectedIcon = AppIcon.allCases.first(where: { $0.alternateName == currentAlternateName }) ?? .default
    }

    var icons: [AppIcon] { AppIcon.allCases }

    func iconTitle(for icon: AppIcon) -> String {
        icon.localizedTitleKey.localizedString
    }

    func requestChange(to icon: AppIcon) {
        guard icon != selectedIcon else { return }
        pendingIcon = icon
    }

    func cancelChange() {
        pendingIcon = nil
    }

    func confirmChange() async {
        guard let icon = pendingIcon else { return }
        pendingIcon = nil
        do {
            try await UIApplication.shared.setAlternateIconName(icon.alternateName)
            selectedIcon = icon
            appSettings.appIcon = icon
        } catch {
            errorMessage = error.localizedDescription
        }
    }
}

// MARK: - Navigation
extension AppIconViewModel {
    func navigateBack() {
        if !path.isEmpty { path.removeLast() }
    }
}
#endif
```

- [ ] **Step 2: Build the Settings target**

```bash
cd /home/alexis/Apps/nym/nym-vpn-client/nym-vpn-apple/Settings && swift build 2>&1 | tail -20
```
Expected: succeeds (or note if env limit prevents it).

- [ ] **Step 3: Commit**

```bash
cd /home/alexis/Apps/nym/nym-vpn-client
git add nym-vpn-apple/Settings/Sources/Settings/Theme/AppIcon/AppIconViewModel.swift
git commit -m "feat(ios): add AppIconViewModel"
```

---

## Task 6: Create `AppIconView`

**Files:**
- Create: `nym-vpn-apple/Settings/Sources/Settings/Theme/AppIcon/AppIconView.swift`

- [ ] **Step 1: Create the file**

The view uses `SettingButton` (the same radio-button-style component used by `DisplayThemeView`), laid out in a `LazyVGrid` with two columns, to show icon previews. The icon preview image comes from the app's main bundle asset catalog via `Image(icon.previewAssetName)` — this works because `AppIcon.previewAssetName` returns the appiconset name, and iOS exposes the 1024×1024 asset under that name.

```swift
#if os(iOS)
import SwiftUI
import AppSettings
import Theme
import UIComponents

public struct AppIconView: View {
    @ObservedObject private var viewModel: AppIconViewModel

    public init(viewModel: AppIconViewModel) {
        self.viewModel = viewModel
    }

    public var body: some View {
        VStack(spacing: 0) {
            navbar()
            iconGrid()
                .frame(maxWidth: MagicNumbers.maxWidth)
            Spacer()
        }
        .navigationBarBackButtonHidden(true)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .ignoresSafeArea(edges: [.bottom])
        .background {
            Color.Nym.surfaceBg
                .ignoresSafeArea()
        }
        .alert(
            LocalizedStringKey("settings.appIcon.confirmTitle"),
            isPresented: Binding(
                get: { viewModel.pendingIcon != nil },
                set: { if !$0 { viewModel.cancelChange() } }
            ),
            actions: {
                Button(LocalizedStringKey("settings.appIcon.confirmAction")) {
                    Task { await viewModel.confirmChange() }
                }
                Button(LocalizedStringKey("settings.appIcon.cancel"), role: .cancel) {
                    viewModel.cancelChange()
                }
            },
            message: {
                Text(LocalizedStringKey("settings.appIcon.confirmBody"))
            }
        )
    }
}

private extension AppIconView {
    @ViewBuilder
    func navbar() -> some View {
        CustomNavBar(
            title: viewModel.title,
            leftButton: CustomNavBarButton(type: .back, action: { viewModel.navigateBack() })
        )
    }

    @ViewBuilder
    func iconGrid() -> some View {
        LazyVGrid(
            columns: [GridItem(.flexible()), GridItem(.flexible())],
            spacing: 16
        ) {
            ForEach(viewModel.icons) { icon in
                iconCard(icon)
                    .onTapGesture { viewModel.requestChange(to: icon) }
            }
        }
        .padding(EdgeInsets(top: 24, leading: 16, bottom: 0, trailing: 16))
    }

    @ViewBuilder
    func iconCard(_ icon: AppIcon) -> some View {
        VStack(spacing: 8) {
            Image(icon.previewAssetName)
                .resizable()
                .aspectRatio(contentMode: .fit)
                .frame(width: 80, height: 80)
                .cornerRadius(18)
                .overlay(
                    RoundedRectangle(cornerRadius: 18)
                        .inset(by: 1.5)
                        .stroke(
                            icon == viewModel.selectedIcon ? Color.Nym.brandPrimary : Color.clear,
                            lineWidth: 3
                        )
                )
            Text(viewModel.iconTitle(for: icon))
                .foregroundStyle(Color.Nym.textPrimary)
                .textStyle(.Body.Large.regular)
        }
        .padding()
        .contentShape(Rectangle())
    }
}
#endif
```

- [ ] **Step 2: Build the Settings target again**

```bash
cd /home/alexis/Apps/nym/nym-vpn-client/nym-vpn-apple/Settings && swift build 2>&1 | tail -20
```

- [ ] **Step 3: Commit**

```bash
cd /home/alexis/Apps/nym/nym-vpn-client
git add nym-vpn-apple/Settings/Sources/Settings/Theme/AppIcon/AppIconView.swift
git commit -m "feat(ios): add AppIconView"
```

---

## Task 7: Wire `SettingsFlowCoordinator`

**Files:**
- Modify: `nym-vpn-apple/Settings/Sources/Settings/SettingsFlowCoordinator.swift`

- [ ] **Step 1: Add `case .appIcon` to the `linkDestination` switch**

Find the switch in `linkDestination`. After the `case .displayTheme:` block:

```swift
        case .displayTheme:
            displayThemeDestination()
```

Add (inside `#if os(iOS)` because `SettingLink.appIcon` is iOS-only):

```swift
#if os(iOS)
        case .appIcon:
            appIconDestination()
#endif
```

- [ ] **Step 2: Add `appIconDestination()` helper to the private extension**

In the `private extension SettingsFlowCoordinator` at the bottom of the file, after `displayThemeDestination()`:

```swift
#if os(iOS)
    @ViewBuilder
    func appIconDestination() -> some View {
        AppIconView(
            viewModel: AppIconViewModel(
                path: $flowState.path,
                appSettings: AppSettings.shared
            )
        )
    }
#endif
```

- [ ] **Step 3: Build Settings**

```bash
cd /home/alexis/Apps/nym/nym-vpn-client/nym-vpn-apple/Settings && swift build 2>&1 | tail -20
```

- [ ] **Step 4: Commit**

```bash
cd /home/alexis/Apps/nym/nym-vpn-client
git add nym-vpn-apple/Settings/Sources/Settings/SettingsFlowCoordinator.swift
git commit -m "feat(ios): register appIcon route in SettingsFlowCoordinator"
```

---

## Task 8: Create alternate icon asset directories and placeholder PNGs

**Files:**
- Create: `nym-vpn-apple/NymVPN/Assets.xcassets/AppIcon-Dark.appiconset/Contents.json`
- Create: `nym-vpn-apple/NymVPN/Assets.xcassets/AppIcon-Dark.appiconset/AppIcon-Dark.png`
- Create: `nym-vpn-apple/NymVPN/Assets.xcassets/AppIcon-Calculator.appiconset/Contents.json`
- Create: `nym-vpn-apple/NymVPN/Assets.xcassets/AppIcon-Calculator.appiconset/AppIcon-Calculator.png`
- Create: `nym-vpn-apple/NymVPN/Assets.xcassets/AppIcon-Notes.appiconset/Contents.json`
- Create: `nym-vpn-apple/NymVPN/Assets.xcassets/AppIcon-Notes.appiconset/AppIcon-Notes.png`

- [ ] **Step 1: Create directories**

```bash
mkdir -p /home/alexis/Apps/nym/nym-vpn-client/nym-vpn-apple/NymVPN/Assets.xcassets/AppIcon-Dark.appiconset
mkdir -p /home/alexis/Apps/nym/nym-vpn-client/nym-vpn-apple/NymVPN/Assets.xcassets/AppIcon-Calculator.appiconset
mkdir -p /home/alexis/Apps/nym/nym-vpn-client/nym-vpn-apple/NymVPN/Assets.xcassets/AppIcon-Notes.appiconset
```

- [ ] **Step 2: Write `Contents.json` for Dark**

```json
{
  "images" : [
    {
      "filename" : "AppIcon-Dark.png",
      "idiom" : "universal",
      "platform" : "ios",
      "size" : "1024x1024"
    }
  ],
  "info" : {
    "author" : "xcode",
    "version" : 1
  }
}
```
File: `NymVPN/Assets.xcassets/AppIcon-Dark.appiconset/Contents.json`

- [ ] **Step 3: Write `Contents.json` for Calculator**

```json
{
  "images" : [
    {
      "filename" : "AppIcon-Calculator.png",
      "idiom" : "universal",
      "platform" : "ios",
      "size" : "1024x1024"
    }
  ],
  "info" : {
    "author" : "xcode",
    "version" : 1
  }
}
```
File: `NymVPN/Assets.xcassets/AppIcon-Calculator.appiconset/Contents.json`

- [ ] **Step 4: Write `Contents.json` for Notes**

```json
{
  "images" : [
    {
      "filename" : "AppIcon-Notes.png",
      "idiom" : "universal",
      "platform" : "ios",
      "size" : "1024x1024"
    }
  ],
  "info" : {
    "author" : "xcode",
    "version" : 1
  }
}
```
File: `NymVPN/Assets.xcassets/AppIcon-Notes.appiconset/Contents.json`

- [ ] **Step 5: Generate placeholder 1024×1024 PNGs**

Try ImageMagick first:
```bash
magick -size 1024x1024 xc:'#0F1A2A' /home/alexis/Apps/nym/nym-vpn-client/nym-vpn-apple/NymVPN/Assets.xcassets/AppIcon-Dark.appiconset/AppIcon-Dark.png && \
magick -size 1024x1024 xc:'#1A1F2E' /home/alexis/Apps/nym/nym-vpn-client/nym-vpn-apple/NymVPN/Assets.xcassets/AppIcon-Calculator.appiconset/AppIcon-Calculator.png && \
magick -size 1024x1024 xc:'#F5C84B' /home/alexis/Apps/nym/nym-vpn-client/nym-vpn-apple/NymVPN/Assets.xcassets/AppIcon-Notes.appiconset/AppIcon-Notes.png
```

If `magick` is not found, try `convert`:
```bash
convert -size 1024x1024 xc:'#0F1A2A' /home/alexis/Apps/nym/nym-vpn-client/nym-vpn-apple/NymVPN/Assets.xcassets/AppIcon-Dark.appiconset/AppIcon-Dark.png
convert -size 1024x1024 xc:'#1A1F2E' /home/alexis/Apps/nym/nym-vpn-client/nym-vpn-apple/NymVPN/Assets.xcassets/AppIcon-Calculator.appiconset/AppIcon-Calculator.png
convert -size 1024x1024 xc:'#F5C84B' /home/alexis/Apps/nym/nym-vpn-client/nym-vpn-apple/NymVPN/Assets.xcassets/AppIcon-Notes.appiconset/AppIcon-Notes.png
```

If neither is available, fall back to Python:
```bash
python3 -c "
from PIL import Image
Image.new('RGB', (1024,1024), (15, 26, 42)).save('/home/alexis/Apps/nym/nym-vpn-client/nym-vpn-apple/NymVPN/Assets.xcassets/AppIcon-Dark.appiconset/AppIcon-Dark.png')
Image.new('RGB', (1024,1024), (26, 31, 46)).save('/home/alexis/Apps/nym/nym-vpn-client/nym-vpn-apple/NymVPN/Assets.xcassets/AppIcon-Calculator.appiconset/AppIcon-Calculator.png')
Image.new('RGB', (1024,1024), (245, 200, 75)).save('/home/alexis/Apps/nym/nym-vpn-client/nym-vpn-apple/NymVPN/Assets.xcassets/AppIcon-Notes.appiconset/AppIcon-Notes.png')
"
```

If PIL is not available, use this raw-bytes Python fallback that writes a minimal valid PNG without external dependencies:
```bash
python3 << 'EOF'
import zlib, struct

def make_png(r, g, b, size=1024):
    def chunk(name, data):
        c = zlib.crc32(name + data) & 0xffffffff
        return struct.pack('>I', len(data)) + name + data + struct.pack('>I', c)
    sig = b'\x89PNG\r\n\x1a\n'
    ihdr = chunk(b'IHDR', struct.pack('>IIBBBBB', size, size, 8, 2, 0, 0, 0))
    row = b'\x00' + bytes([r, g, b]) * size
    raw = row * size
    idat = chunk(b'IDAT', zlib.compress(raw))
    iend = chunk(b'IEND', b'')
    return sig + ihdr + idat + iend

icons = [
    ('/home/alexis/Apps/nym/nym-vpn-client/nym-vpn-apple/NymVPN/Assets.xcassets/AppIcon-Dark.appiconset/AppIcon-Dark.png', 0x0F, 0x1A, 0x2A),
    ('/home/alexis/Apps/nym/nym-vpn-client/nym-vpn-apple/NymVPN/Assets.xcassets/AppIcon-Calculator.appiconset/AppIcon-Calculator.png', 0x1A, 0x1F, 0x2E),
    ('/home/alexis/Apps/nym/nym-vpn-client/nym-vpn-apple/NymVPN/Assets.xcassets/AppIcon-Notes.appiconset/AppIcon-Notes.png', 0xF5, 0xC8, 0x4B),
]
for path, r, g, b in icons:
    with open(path, 'wb') as f:
        f.write(make_png(r, g, b))
    print(f'Wrote {path}')
EOF
```

- [ ] **Step 6: Verify the PNGs exist and are non-zero**

```bash
ls -lh /home/alexis/Apps/nym/nym-vpn-client/nym-vpn-apple/NymVPN/Assets.xcassets/AppIcon-Dark.appiconset/AppIcon-Dark.png \
       /home/alexis/Apps/nym/nym-vpn-client/nym-vpn-apple/NymVPN/Assets.xcassets/AppIcon-Calculator.appiconset/AppIcon-Calculator.png \
       /home/alexis/Apps/nym/nym-vpn-client/nym-vpn-apple/NymVPN/Assets.xcassets/AppIcon-Notes.appiconset/AppIcon-Notes.png
```
Expected: three files, each > 1 KB.

- [ ] **Step 7: Commit**

```bash
cd /home/alexis/Apps/nym/nym-vpn-client
git add nym-vpn-apple/NymVPN/Assets.xcassets/AppIcon-Dark.appiconset/ \
        nym-vpn-apple/NymVPN/Assets.xcassets/AppIcon-Calculator.appiconset/ \
        nym-vpn-apple/NymVPN/Assets.xcassets/AppIcon-Notes.appiconset/
git commit -m "feat(ios): add alternate app icon appiconsets (placeholder PNGs)"
```

---

## Task 9: Configure Xcode build settings for alternate icons

**Files:**
- Modify: `nym-vpn-apple/NymVPN.xcodeproj/project.pbxproj`

The iOS NymVPN app target has two build configurations:
- Debug:   `D99A14852B357E9A00F2728B`
- Release: `D99A14862B357E9A00F2728B`

Both already have `ASSETCATALOG_COMPILER_INCLUDE_ALL_APPICON_ASSETS = YES;` but lack `ASSETCATALOG_COMPILER_ALTERNATE_APPICON_NAMES`.

- [ ] **Step 1: Add `ASSETCATALOG_COMPILER_ALTERNATE_APPICON_NAMES` to the Debug config (ID `D99A14852B357E9A00F2728B`)**

Find this exact string in the file:
```
			D99A14852B357E9A00F2728B /* Debug */ = {
				isa = XCBuildConfiguration;
				buildSettings = {
					ASSETCATALOG_COMPILER_APPICON_NAME = AppIcon;
					ASSETCATALOG_COMPILER_GLOBAL_ACCENT_COLOR_NAME = AccentColor;
					ASSETCATALOG_COMPILER_INCLUDE_ALL_APPICON_ASSETS = YES;
```

Replace `ASSETCATALOG_COMPILER_INCLUDE_ALL_APPICON_ASSETS = YES;` with:
```
					ASSETCATALOG_COMPILER_ALTERNATE_APPICON_NAMES = "AppIcon-Dark AppIcon-Calculator AppIcon-Notes";
					ASSETCATALOG_COMPILER_INCLUDE_ALL_APPICON_ASSETS = YES;
```

- [ ] **Step 2: Add the same key to the Release config (ID `D99A14862B357E9A00F2728B`)**

Find:
```
			D99A14862B357E9A00F2728B /* Release */ = {
				isa = XCBuildConfiguration;
				buildSettings = {
					ASSETCATALOG_COMPILER_APPICON_NAME = AppIcon;
					ASSETCATALOG_COMPILER_GLOBAL_ACCENT_COLOR_NAME = AccentColor;
					ASSETCATALOG_COMPILER_INCLUDE_ALL_APPICON_ASSETS = YES;
```

Replace `ASSETCATALOG_COMPILER_INCLUDE_ALL_APPICON_ASSETS = YES;` with:
```
					ASSETCATALOG_COMPILER_ALTERNATE_APPICON_NAMES = "AppIcon-Dark AppIcon-Calculator AppIcon-Notes";
					ASSETCATALOG_COMPILER_INCLUDE_ALL_APPICON_ASSETS = YES;
```

- [ ] **Step 3: Verify the edit**

```bash
grep -n "ASSETCATALOG_COMPILER_ALTERNATE_APPICON_NAMES" /home/alexis/Apps/nym/nym-vpn-client/nym-vpn-apple/NymVPN.xcodeproj/project.pbxproj
```
Expected: 2 lines, one for Debug and one for Release.

- [ ] **Step 4: Commit**

```bash
cd /home/alexis/Apps/nym/nym-vpn-client
git add nym-vpn-apple/NymVPN.xcodeproj/project.pbxproj
git commit -m "feat(ios): register alternate icon names in Xcode build settings"
```

---

## Task 10: Add localized strings

**Files:**
- Modify: `nym-vpn-apple/NymVPN/Resources/Localizable.xcstrings`

The `.xcstrings` file is a JSON dictionary. New entries only need English (state `"translated"`) at this stage — translators will fill in other languages. The file ends with `"version" : "1.0"\n}`, so insert before the closing `}` of the outer `"strings"` dict.

- [ ] **Step 1: Insert the 9 new keys**

Find the end of the strings dict (currently ends with the `"Yes"` key block around line 36839). The insertion point is just before `  },\n  "version" : "1.0"`. Add the following block **before** `  },\n  "version" : "1.0"`:

```json
    "settings.appIcon.cancel" : {
      "extractionState" : "manual",
      "localizations" : {
        "en" : {
          "stringUnit" : {
            "state" : "translated",
            "value" : "Cancel"
          }
        }
      }
    },
    "settings.appIcon.calculator" : {
      "extractionState" : "manual",
      "localizations" : {
        "en" : {
          "stringUnit" : {
            "state" : "translated",
            "value" : "Calculator"
          }
        }
      }
    },
    "settings.appIcon.confirmAction" : {
      "extractionState" : "manual",
      "localizations" : {
        "en" : {
          "stringUnit" : {
            "state" : "translated",
            "value" : "Change icon"
          }
        }
      }
    },
    "settings.appIcon.confirmBody" : {
      "extractionState" : "manual",
      "localizations" : {
        "en" : {
          "stringUnit" : {
            "state" : "translated",
            "value" : "iOS will show a system alert confirming the change. Your home-screen icon will refresh momentarily."
          }
        }
      }
    },
    "settings.appIcon.confirmTitle" : {
      "extractionState" : "manual",
      "localizations" : {
        "en" : {
          "stringUnit" : {
            "state" : "translated",
            "value" : "Change app icon?"
          }
        }
      }
    },
    "settings.appIcon.dark" : {
      "extractionState" : "manual",
      "localizations" : {
        "en" : {
          "stringUnit" : {
            "state" : "translated",
            "value" : "Dark"
          }
        }
      }
    },
    "settings.appIcon.default" : {
      "extractionState" : "manual",
      "localizations" : {
        "en" : {
          "stringUnit" : {
            "state" : "translated",
            "value" : "Default"
          }
        }
      }
    },
    "settings.appIcon.notes" : {
      "extractionState" : "manual",
      "localizations" : {
        "en" : {
          "stringUnit" : {
            "state" : "translated",
            "value" : "Notes"
          }
        }
      }
    },
    "settings.appIcon.title" : {
      "extractionState" : "manual",
      "localizations" : {
        "en" : {
          "stringUnit" : {
            "state" : "translated",
            "value" : "App icon"
          }
        }
      }
    }
```

Insert this immediately **before** the last `  },` that closes the `"strings"` dict (the one followed immediately by `  "version" : "1.0"`).

Concretely the file's last lines currently look like:
```
        }
      }
    }
  },
  "version" : "1.0"
}
```

After the edit, the last section should be:
```
        }
      }
    },
    "settings.appIcon.cancel" : {
      ...
    },
    ...
    "settings.appIcon.title" : {
      ...
      }
    }
  },
  "version" : "1.0"
}
```

Note the `,` that must be added after the last pre-existing key's closing `}` to keep the JSON valid.

- [ ] **Step 2: Validate the JSON is still valid**

```bash
python3 -c "import json; json.load(open('/home/alexis/Apps/nym/nym-vpn-client/nym-vpn-apple/NymVPN/Resources/Localizable.xcstrings')); print('JSON valid')"
```
Expected: `JSON valid`

- [ ] **Step 3: Commit**

```bash
cd /home/alexis/Apps/nym/nym-vpn-client
git add nym-vpn-apple/NymVPN/Resources/Localizable.xcstrings
git commit -m "feat(ios): add settings.appIcon.* localization keys (English)"
```

---

## Task 11: Full build attempt and final commit

- [ ] **Step 1: Attempt xcodebuild**

```bash
cd /home/alexis/Apps/nym/nym-vpn-client/nym-vpn-apple && \
xcodebuild -scheme NymVPN -destination "generic/platform=iOS" -configuration Debug -quiet build 2>&1 | tail -30
```
If Xcode is not installed, this will fail with "xcodebuild: command not found" — note this and continue.

- [ ] **Step 2: Attempt Settings swift build as a syntax gate**

```bash
cd /home/alexis/Apps/nym/nym-vpn-client/nym-vpn-apple/Settings && swift build 2>&1 | tail -20
```

- [ ] **Step 3: Final consolidating commit if everything is clean**

```bash
cd /home/alexis/Apps/nym/nym-vpn-client
git log --oneline -10
```

If all previous per-task commits exist, no additional commit is needed. If any prior task skipped its commit, do a final sweep:

```bash
git status nym-vpn-apple/
```

And commit any remaining staged changes.
