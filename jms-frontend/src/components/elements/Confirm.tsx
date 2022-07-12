import React from "react";
import { Button, Modal } from "react-bootstrap";

import { createConfirmation, confirmable, ReactConfirmProps } from 'react-confirm';

type ConfirmModalOptions<T> = {
  title?: React.ReactNode,

  // Either this....
  okBtn?: Omit<React.ComponentProps<Button>, "onClick">,
  cancelBtn?: Omit<React.ComponentProps<Button>, "onClick">,
  
  // Or this...
  render?: (ok: (data?: T) => void, cancel: () => void) => React.ReactNode,
  
  // Or this...
  okText?: React.ReactNode,
  cancelText?: React.ReactNode,
  okVariant?: string,
  cancelVariant?: string,
} & Omit<React.ComponentProps<Modal>, "show" | "onHide">;

type ConfirmModalProps<T> = ReactConfirmProps & ConfirmModalOptions<T>;

export class ConfirmModal extends React.PureComponent<ConfirmModalProps<any>> {
  render() {
    const { className, title, confirmation, show, proceed, cancel, dismiss, render, okBtn, cancelBtn, okText, cancelText, okVariant, cancelVariant, ...props } = this.props;

    const okFn = (data?: any) => proceed(JSON.stringify(data === undefined ? {} : data));
    const cancelFn = () => cancel();

    return <Modal
      show={show}
      onHide={cancelFn}
      backdrop="static"
      centered
      { ...props }
    >
      {
        title ? <Modal.Header> <Modal.Title> { title } </Modal.Title> </Modal.Header> : undefined
      }
      {
        render ? render(okFn, cancelFn) : <React.Fragment>
          <Modal.Body> { confirmation || "Are you sure?" } </Modal.Body>
          <Modal.Footer>
            {
              okBtn ? <Button {...okBtn} onClick={() => okFn()} /> : 
              <Button
                onClick={() => okFn()}
                variant={okVariant || "success"}
              >
                { okText || "OK" }
              </Button>
            }

            {
              cancelBtn ? <Button {...cancelBtn} onClick={() => cancelFn()} /> : 
              <Button
                onClick={() => cancelFn()}
                variant={cancelVariant || "secondary"}
              >
                { cancelText || "Cancel" }
              </Button>
            }
          </Modal.Footer>
        </React.Fragment>
      }
    </Modal>
  }
}

const confirmBackend = createConfirmation(confirmable(ConfirmModal));

export async function confirmModal<T>(confirmation: ReactConfirmProps["confirmation"], options?: ConfirmModalOptions<T>) {
  let data = await confirmBackend({ confirmation, ...(options || {}) });
  return JSON.parse(data) as T;
}

export default async function confirmBool(confirmation: ReactConfirmProps["confirmation"], options?: ConfirmModalOptions<{}>) {
  try {
    await confirmBackend({ confirmation, ...(options || {}) });
    return true;
  } catch {
    return false;
  }
}

export async function withConfirm(fn: () => void, confirmation?: ReactConfirmProps["confirmation"], options?: ConfirmModalOptions<{}>) {
  if (await confirmBool(confirmation || "Are you sure?", options || {})) {
    fn();
  }
}