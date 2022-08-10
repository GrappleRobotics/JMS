import React from "react";
import { OverlayTrigger, Tooltip, TooltipProps } from "react-bootstrap";
import { Combine } from "support/util";

type SimpleTooltipProps = Combine<{
  tip: React.ReactNode | string,
  children: React.ReactElement
}, TooltipProps>;

export default class SimpleTooltip extends React.PureComponent<SimpleTooltipProps> {
  render() {
    let { id, tip, children, placement, ...props } = this.props;
    return <OverlayTrigger
      placement={placement || "top"}
      overlay={
        <Tooltip id={id} {...props}>
          { tip }
        </Tooltip>
      }
    >
      { children }
    </OverlayTrigger>
  }
}