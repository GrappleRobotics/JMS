import React from "react";
import { OverlayTrigger, Tooltip, TooltipProps } from "react-bootstrap";
import { Combine } from "@/app/support/util";

type SimpleTooltipProps = Combine<{
  id: string,
  tip: React.ReactNode | string,
  children: React.ReactNode,
  disabled?: boolean
}, TooltipProps>;

export default class SimpleTooltip extends React.PureComponent<SimpleTooltipProps> {
  render() {
    let { id, tip, children, placement, disabled, ...props } = this.props;
    return disabled ? <span> { children } </span> : <OverlayTrigger
      placement={placement || "top"}
      overlay={
        <Tooltip id={id} {...props}>
          { tip }
        </Tooltip>
      }
    >
      <span> { children } </span>
    </OverlayTrigger>
  }
}