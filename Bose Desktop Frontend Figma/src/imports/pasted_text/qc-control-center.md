# MASTER FIGMA PROMPT
# QC CONTROL CENTER — PREMIUM WINDOWS DESKTOP UI

Design and create a complete, production-quality frontend/UI/UX system for:

“QC CONTROL CENTER”

A premium LOCAL Windows desktop control center for Bose QuietComfort headphones.

==================================================
IMPORTANT — FRONTEND ONLY
==================================================

This is strictly a FRONTEND / UI / UX task.

Do NOT implement or simulate actual hardware communication.

Do NOT build:

- Bluetooth
- BLE
- GATT
- RFCOMM
- Rust
- Tauri backend
- Windows Bluetooth APIs
- Bose protocols
- Database
- Cloud services
- Real device communication

The design will later be implemented using:

React
TypeScript
Tauri
Rust

Your responsibility is to create the complete visual system and frontend experience that can later connect cleanly to that backend.

Use realistic mock data only for visualizing the interface.

Never imply that mock data represents verified real hardware behavior.

==================================================
1. PRODUCT VISION
==================================================

Create a premium Windows desktop application for controlling Bose QuietComfort headphones locally.

The application should feel like a professional hardware-control utility rather than a website.

Design personality:

- Premium
- Minimal
- Sophisticated
- Calm
- Technical but approachable
- Fast
- Clean
- Professional
- Native-feeling Windows desktop software

Avoid:

- Cyberpunk
- Neon
- RGB
- Hacker aesthetics
- Excessive glassmorphism
- Overly futuristic interfaces
- Generic SaaS dashboards
- Excessive gradients
- Excessive decoration

Do NOT clone Bose's UI.

Do NOT use Bose logos, copyrighted artwork, or proprietary assets.

Create an original visual identity inspired by premium audio hardware.

The application should communicate:

“Professional control over my headphones.”

==================================================
2. DESIGN PHILOSOPHY
==================================================

Prioritize:

1. Visual hierarchy
2. Usability
3. Information clarity
4. Premium feel
5. Consistency
6. Accessibility
7. Windows desktop conventions
8. Implementation feasibility

IMPORTANT:

Do NOT turn every feature into a separate floating card.

The interface should feel like ONE cohesive application.

Use cards only when they improve grouping or interaction.

Use whitespace, typography, alignment, and hierarchy to create structure.

The dashboard should feel intentional and spacious, not like a collection of widgets.

==================================================
3. VISUAL DIRECTION
==================================================

Primary design direction:

Dark premium desktop utility.

Use:

- Deep charcoal / near-black background
- Slightly lighter surfaces
- Subtle borders
- Soft shadows
- Near-white primary text
- Muted secondary text
- Restrained cool blue / indigo accent
- Subtle green for success
- Amber for warnings
- Soft red for errors

Do not overuse accent colors.

The visual language should be restrained.

Think:

premium audio hardware + modern Windows utility

rather than:

gaming dashboard + futuristic AI interface.

==================================================
4. LIGHT MODE + DARK MODE
==================================================

Create both:

- Dark mode
- Light mode
- System theme

Dark mode is the primary presentation.

Light mode should not simply invert the dark design.

Create a properly designed light theme with:

- warm/light neutral background
- white surfaces
- dark typography
- subtle borders
- restrained blue accent

All components must support both themes.

==================================================
5. DESIGN SYSTEM
==================================================

Create a complete reusable design system.

Define:

- Colors
- Typography
- Spacing
- Border radius
- Shadows
- Borders
- Elevation
- Transitions
- Component states

Use a consistent spacing system.

Use consistent corner radii.

Use consistent iconography.

Do not hardcode random visual values throughout the design.

==================================================
6. TYPOGRAPHY
==================================================

Use a modern highly readable sans-serif font appropriate for Windows desktop software.

Typography hierarchy should include:

- Display / hero
- Page title
- Section heading
- Card heading
- Body
- Secondary text
- Caption
- Status text

Avoid oversized typography that wastes desktop space.

Prioritize readability and information density.

==================================================
7. DESKTOP CANVAS
==================================================

Primary design resolution:

1440 × 900

Also ensure the layout works at:

1280 × 800
1920 × 1080

The application should behave like a desktop application, not a responsive website.

Use:

LEFT SIDEBAR
+
MAIN CONTENT

Optional right-side detail panels may be used when they improve the experience.

==================================================
8. GLOBAL APPLICATION SHELL
==================================================

Create a reusable application shell.

Structure:

┌──────────────────────────────────────────────────┐
│                                                  │
│  SIDEBAR              MAIN CONTENT               │
│                                                  │
│                                                  │
│                                                  │
│                                                  │
│                                                  │
│                                                  │
└──────────────────────────────────────────────────┘

Sidebar:

- App icon
- QC CONTROL CENTER
- Dashboard
- Device
- Noise Control
- Equalizer
- Profiles
- Diagnostics
- Settings

Bottom of sidebar:

- Connection status
- Battery status

The sidebar should be compact and elegant.

Do not make it oversized.

Active navigation should be obvious but subtle.

==================================================
9. DASHBOARD — PRIMARY SCREEN
==================================================

The dashboard is the most important screen.

It should NOT feel like a generic grid of six cards.

Create a strong visual composition.

The headphone/device visual should be the visual anchor.

Use an original abstract headphone illustration, silhouette, or tasteful generated visual.

Do NOT use an official Bose product image unless legally supplied.

Suggested composition:

--------------------------------------------------
Bose QuietComfort
Connected
Battery 78%

        [HEADPHONE VISUAL]

Noise Control
[ Quiet ] [ Aware ] [ Custom ]

Equalizer
Bass ─────────●────
Mid  ───────●──────
Treble ───────●────

Volume ─────────●──

Quick Profiles
Music   Gaming   Study
--------------------------------------------------

The dashboard should prioritize the most important actions:

1. Device status
2. Noise control
3. Volume
4. EQ
5. Profiles

Do not overload the dashboard.

Use secondary details elsewhere.

==================================================
10. DEVICE VISUAL
==================================================

Create an elegant visual representation of the headphones.

Options:

- Original headphone illustration
- Minimal silhouette
- Abstract device rendering
- Premium line-art representation

The visual should feel premium but not dominate the entire interface.

Use it as the visual anchor of the dashboard.

==================================================
11. CAPABILITY-AWARE DESIGN
==================================================

This is a core requirement.

Hardware-specific controls must support these states:

VERIFIED
SUPPORTED
UNKNOWN
EXPERIMENTAL
UNSUPPORTED

Create reusable capability/status badges.

Suggested treatment:

VERIFIED
Subtle green

SUPPORTED
Subtle blue

UNKNOWN
Neutral gray

EXPERIMENTAL
Subtle amber

UNSUPPORTED
Muted/disabled

Do NOT rely on color alone.

Include text/icon indicators.

Example:

Noise Control
UNKNOWN

“Not yet verified on your headphones.”

SUPPORTED:

“Available through the detected device interface.”

EXPERIMENTAL:

“Not fully verified on this device.”

UNSUPPORTED:

“This feature is not exposed through the available device interface.”

The UI must never make an unsupported feature look broken.

==================================================
12. DEVICE PAGE
==================================================

Create a complete Device page.

Header:

Bose QuietComfort
Connected

Primary information:

Connection
Connected

Transport
Bluetooth

Battery
78%

Device name
Bose QuietComfort

Device ID
••••••••

Firmware
Unavailable

Actions:

Reconnect
Disconnect
Run Diagnostics

Also create a connection timeline:

Connected
Disconnected
Reconnected

Keep this page information-rich but clean.

==================================================
13. NOISE CONTROL PAGE
==================================================

Create a dedicated Noise Control experience.

Primary modes:

Quiet
Aware
Custom

Use a premium segmented control.

Example:

[ QUIET ] [ AWARE ] [ CUSTOM ]

If Custom is selected:

Noise Cancellation
0 ─────────────●──── 10

Wind Block
[ ON ]

IMPORTANT:

These controls must support capability states.

If UNKNOWN:

Disable interaction.

Show:

“Not yet verified on your headphones.”

Do NOT visually imply that the control actually changes the headphones.

==================================================
14. EQUALIZER PAGE
==================================================

Create a premium EQ experience.

Controls:

Bass
Mid
Treble

Example:

Bass
−10 ─────────●──────── +10

Mid
−10 ───────────●────── +10

Treble
−10 ───────●────────── +10

Presets:

Flat
Music
Bass Boost
Podcast
Gaming
Custom

Create two clearly separated sections:

HARDWARE EQ

Controls headphone hardware functionality.

SOFTWARE EQ

Applies processing through the Windows audio pipeline.

The two must never look like the same technology.

Hardware EQ should have its capability status visible.

==================================================
15. PROFILES
==================================================

Create a premium profile manager.

Default profiles:

Music
Gaming
Study
Podcast
Custom

Each profile should show:

- Name
- EQ values
- Noise mode
- Last used
- Apply action

Example:

Music

Bass +4
Mid 0
Treble +2

Noise
Quiet

[ Apply ]

Create:

+ New Profile

Profile editor:

Name
Bass
Mid
Treble
Noise Control
Wind Block

[ Save Profile ]

Profiles are LOCAL UI concepts and do not require cloud functionality.

==================================================
16. DIAGNOSTICS PAGE
==================================================

Create a polished technical diagnostics interface.

Important:

It should look like professional engineering software.

It should NOT look like:

- Hacker terminal
- Cyberpunk dashboard
- Raw developer console

Header:

Diagnostics

Device:
Bose QuietComfort

Connection:
Connected

Sections:

Bluetooth
BLE
Services
Characteristics
Battery
Audio
Capabilities

Capability table:

Feature             Status

Battery             VERIFIED
Volume              SUPPORTED
Noise Control       UNKNOWN
Aware Mode          UNKNOWN
EQ                  UNKNOWN
Custom ANC          UNKNOWN

Use reusable capability badges.

Actions:

Start Capture
Stop Capture
Export Report

==================================================
17. DISCOVERY SESSION
==================================================

Create a diagnostic event timeline.

Example:

08:42:13
Device notification received

08:42:15
USER ACTION
ANC_CHANGE

08:42:15
Characteristic updated

08:42:18
Device state changed

Use:

- timestamps
- event type
- concise description
- status indicator

Keep the design technical but elegant.

==================================================
18. SETTINGS
==================================================

Create a complete Settings experience.

GENERAL

Start with Windows
Toggle

Start minimized
Toggle

Minimize to tray
Toggle

Auto-connect
Toggle


APPEARANCE

System
Light
Dark


DEVICE

Preferred Device

Bose QuietComfort

Auto reconnect

Reconnect automatically


NOTIFICATIONS

Device connected
Toggle

Device disconnected
Toggle

Low battery
Toggle


DIAGNOSTICS

Logging level

Off
Error
Warn
Info
Debug
Trace

Export Logs

Clear Logs

Use grouped settings sections instead of excessive cards.

==================================================
19. SYSTEM TRAY
==================================================

Design a Windows-style tray popup.

Example:

--------------------------------
Bose QC

● Connected

Battery
78%

Noise Control
Quiet

Volume
65%

--------------------------------

Open Control Center
Reconnect
Disconnect

--------------------------------

Settings
Exit
--------------------------------

Keep it compact.

It should look like a real desktop tray utility.

==================================================
20. CONNECTION STATES
==================================================

Design complete states for:

1. Connected
2. Disconnected
3. Connecting
4. Discovering
5. Reconnecting
6. Error
7. Bluetooth disabled
8. Device unavailable
9. Simulated

CONNECTED:

Bose QuietComfort
Connected

DISCONNECTED:

“No Bose headphones connected.”

[ Connect ]

CONNECTING:

“Connecting to Bose QuietComfort…”

Use subtle loading animation.

BLUETOOTH DISABLED:

“Bluetooth is turned off in Windows.”

[ Open Bluetooth Settings ]

DEVICE UNAVAILABLE:

“Your headphones are out of range or powered off.”

SIMULATED:

“SIMULATED DEVICE”

The simulated state must be visually unmistakable.

==================================================
21. BATTERY STATES
==================================================

Design:

100%
78%
50%
20%
10%
Critical

States:

Normal
Low
Critical

Use restrained warnings.

Do not make low battery states visually alarming.

==================================================
22. SIMULATED DEVICE
==================================================

Create a complete simulated-device experience.

Example:

SIMULATED DEVICE

Bose QuietComfort

Connected

Battery
78%

This is a development/testing state.

It must be clearly labelled everywhere necessary.

Never make mock data indistinguishable from verified hardware data.

==================================================
23. EXPERIMENTAL FEATURES
==================================================

Create an EXPERIMENTAL badge/state.

Example:

EXPERIMENTAL

“Not fully verified on your device.”

Experimental controls may be visible but should clearly communicate uncertainty.

Do not use alarming styling.

Keep the design consistent with the rest of the application.

==================================================
24. NOTIFICATIONS / TOASTS
==================================================

Create reusable toast notifications.

CONNECTED

“Bose QuietComfort connected.”

DISCONNECTED

“Bose QuietComfort disconnected.”

LOW BATTERY

“Headphones battery is low.”

SUCCESS

“Noise Control changed to Quiet.”

UNVERIFIED

“Command sent, but device state could not be verified.”

ERROR

“Unable to communicate with the headphones.”

Create success, warning, error, info variants.

==================================================
25. MODALS
==================================================

Create reusable modals:

- Device Details
- Disconnect Confirmation
- Create Profile
- Edit Profile
- Export Diagnostics
- Clear Logs
- Experimental Feature Warning

Use consistent modal sizing and spacing.

Do not overuse modal dialogs.

==================================================
26. EMPTY STATES
==================================================

Create polished empty states.

NO DEVICE:

“No Bose headphones connected.”

[ Open Bluetooth Settings ]

NO PROFILES:

“You haven't created any profiles yet.”

[ Create Profile ]

NO DIAGNOSTICS:

“No diagnostic sessions yet.”

[ Start Discovery ]

Use useful illustrations or subtle icons.

Avoid giant empty screens.

==================================================
27. ACCESSIBILITY
==================================================

Design with accessibility in mind.

Include:

- strong contrast
- keyboard focus states
- hover states
- active states
- disabled states
- readable typography
- sufficiently large click targets
- clear keyboard navigation

Do not communicate status using color alone.

==================================================
28. MICRO-INTERACTIONS
==================================================

Use subtle animations for:

- sidebar navigation
- page transitions
- slider interaction
- button press
- connection changes
- battery updates
- toast appearance
- modal opening
- loading states

Animation should feel:

- fast
- subtle
- premium

Do NOT over-animate.

==================================================
29. COMPONENT LIBRARY
==================================================

Create reusable components and variants:

AppShell
Sidebar
TopBar
DeviceCard
DeviceVisual
BatteryIndicator
ConnectionBadge
CapabilityBadge
StatusBadge
Slider
SegmentedControl
Toggle
Button
IconButton
Card
ProfileCard
ProfileEditor
DiagnosticTable
DiagnosticEvent
Toast
Modal
EmptyState
LoadingState
ErrorState
TrayMenu

Every reusable component should have appropriate:

- default
- hover
- pressed
- focused
- disabled
- loading
- success
- error
- unknown
- experimental

states where applicable.

==================================================
30. DESIGN TOKENS
==================================================

Create reusable tokens for:

Colors
Typography
Spacing
Radius
Borders
Shadows
Transitions

The final system should be easy to implement in React/Tailwind.

==================================================
31. ICONOGRAPHY
==================================================

Use one consistent modern icon family.

Potential icons:

Headphones
Bluetooth
Battery
Volume
Music
Settings
Activity
Sliders
Shield
Alert
Check
X
Refresh
Power
Chevron
Play
Pause
Next
Previous

Do not mix incompatible icon styles.

==================================================
32. FRONTEND STATE MODEL
==================================================

Design the UI around a future state model such as:

connectionState
device
battery
capabilities
noiseControl
eq
profiles
settings
diagnostics

The design MUST NOT assume every feature exists.

Controls should dynamically respond to capability state.

==================================================
33. COMPLETE COMPONENT STATES
==================================================

For major controls create:

Default
Hover
Pressed
Focused
Disabled
Loading
Success
Error
Unknown
Experimental

For device:

Connected
Disconnected
Connecting
Discovering
Reconnecting
Error
Simulated

For capabilities:

Verified
Supported
Unknown
Experimental
Unsupported

==================================================
34. FINAL SCREENS
==================================================

Create complete polished designs for:

1. Dashboard
2. Device
3. Noise Control
4. Equalizer
5. Profiles
6. Diagnostics
7. Settings

Also create:

8. No Device
9. Connecting
10. Discovering
11. Reconnecting
12. Simulated Device
13. Error
14. Bluetooth Disabled
15. Device Unavailable
16. Tray Popup
17. Profile Creation Modal
18. Device Details Modal
19. Experimental Feature Warning
20. Diagnostic Export Modal

==================================================
35. IMPLEMENTATION-FRIENDLY DESIGN
==================================================

The design will later be implemented in:

React
TypeScript
Tauri
Rust
Tailwind CSS

Therefore:

- Use reusable components
- Use consistent spacing
- Use consistent tokens
- Avoid unnecessarily complex layouts
- Avoid visual elements that are difficult to implement
- Keep interactions clearly defined
- Make component states explicit

The design should translate cleanly from Figma into production frontend code.

==================================================
36. FINAL QUALITY BAR
==================================================

The final result should look like something a professional desktop software company would ship.

Prioritize:

1. Visual hierarchy
2. Usability
3. Consistency
4. Clarity
5. Premium feel
6. Accessibility
7. Capability-aware states
8. Windows desktop conventions
9. Implementation feasibility

Do NOT make every screen a grid of cards.

Do NOT add decorative elements without purpose.

Do NOT sacrifice usability for visual effects.

Use whitespace intelligently.

Maintain a strong visual hierarchy.

==================================================
37. FINAL DELIVERABLE
==================================================

Produce a complete Figma design system and frontend prototype for:

QC CONTROL CENTER

The final Figma project must contain:

- premium desktop shell
- sidebar navigation
- dashboard
- device management
- noise control
- equalizer
- profiles
- diagnostics
- settings
- tray UI
- notifications
- modals
- empty states
- error states
- connection states
- simulated-device state
- capability states
- dark mode
- light mode
- reusable components
- component variants
- design tokens
- accessible interaction states

The result must feel like:

“A premium local Windows control center for Bose QuietComfort headphones.”

It must NOT feel like:

- a website
- a SaaS dashboard
- a hacker interface
- a gaming utility
- a Bose UI clone

==================================================
38. FINAL INSTRUCTION
==================================================

Before finalizing the design, review the entire application as a product designer.

Check:

- Is the dashboard visually balanced?
- Is the headphone visual the main visual anchor?
- Are the most important controls immediately accessible?
- Are capability states obvious?
- Are unsupported features clearly communicated?
- Does the UI remain coherent without hardware?
- Does dark mode feel premium?
- Does light mode feel intentional?
- Are there too many cards?
- Are interactions obvious?
- Can the design realistically be implemented in React/Tauri?
- Does it feel like a real Windows application?

If any screen feels cluttered, simplify it.

If any screen feels empty, improve hierarchy rather than adding unnecessary cards.

The final result should be **premium, restrained, functional, cohesive, and implementation-ready.**