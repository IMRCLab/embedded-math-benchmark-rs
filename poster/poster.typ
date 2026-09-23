#import "theme.typ": divider
#import "panels/1_header.typ": header-panel
#import "panels/2_infrastructure.typ": infrastructure-panel
#import "panels/3_abi_shape.typ": abi-shape-panel
#import "panels/4_build_profile.typ": build-profile-panel
#import "panels/5_math_backend.typ": math-backend-panel
#import "panels/6_caveats.typ": caveats-panel
#import "panels/4b_robotics_pipelines.typ": robotics-pipelines-panel
#import "panels/7_guidance.typ": guidance-panel

#set page("a0", margin: (x: 3cm, top: 3cm, bottom: 3cm))
#set text(size: 22pt, font: "Noto Sans")
#set par(spacing: 0.5em, leading: 0.42em)
#set list(spacing: 0.5em)
#set block(spacing: 0.55em)

#header-panel()
#v(24pt)
#infrastructure-panel()
#divider()
#abi-shape-panel()
#divider()
#build-profile-panel()
#divider()
#math-backend-panel()
#v(20pt)
#caveats-panel()
#divider()
#robotics-pipelines-panel()
#divider()
#guidance-panel()


