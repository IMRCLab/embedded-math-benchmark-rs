#import "theme.typ": divider
#import "panels/1_header.typ": header-panel
#import "panels/2_infrastructure.typ": infrastructure-panel
#import "panels/3_fair_comparison.typ": fair-comparison-panel
#import "panels/5_math_backend.typ": math-backend-panel
#import "panels/6_caveats.typ": open-questions-panel
#import "panels/4b_robotics_pipelines.typ": robotics-pipelines-panel
#import "panels/7_guidance.typ": guidance-panel

#set page("a0", margin: (x: 3cm, top: 2.2cm, bottom: 2.2cm))
#set text(size: 22pt)
#set par(spacing: 0.5em, leading: 0.42em)
#set list(spacing: 0.5em)
#set block(spacing: 0.55em)

#header-panel()
#v(24pt)
#infrastructure-panel()
#divider()
#fair-comparison-panel()
#divider()
#math-backend-panel()
#divider()
#robotics-pipelines-panel()
#divider()
#guidance-panel()
#v(24pt)
#open-questions-panel()


