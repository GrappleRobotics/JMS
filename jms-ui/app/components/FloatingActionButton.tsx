import "./FloatingActionButton.scss"
import { IconProp } from "@fortawesome/fontawesome-svg-core"
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome"
import React from "react"

interface FloatingActionButtonProps {
  icon: IconProp,
  variant: string,
  className?: string,
  onClick?: () => void,
};

export default function FloatingActionButton(props: FloatingActionButtonProps) {
  return <div className={`fab bg-${props.variant} ${props.className}`} onClick={props.onClick ? props.onClick : () => {}}>
    <FontAwesomeIcon icon={props.icon} /> &nbsp;
  </div>
}