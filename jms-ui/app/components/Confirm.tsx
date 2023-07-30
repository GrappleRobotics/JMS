import React from "react";
import { Button, Modal } from "react-bootstrap";

import { createConfirmation, confirmable, ReactConfirmProps } from 'react-confirm';

type ConfirmModalOptions<T> = {
  title?: React.ReactNode,

  // Either this....
  okBtn?: Omit<React.ComponentProps<typeof Button>, "onClick">,
  cancelBtn?: Omit<React.ComponentProps<typeof Button>, "onClick">,
  
  // Or this...
  render?: (ok: (data?: T) => void, cancel: () => void) => React.ReactNode,
  
  // Or this...
  okText?: React.ReactNode,
  cancelText?: React.ReactNode,
  okVariant?: string,
  cancelVariant?: string,

  cancelIfBackdrop?: boolean,

  renderInner?: (data: T, onUpdate: (data: T) => void, ok: () => void, cancel: () => void) => React.ReactNode,
  data?: T
} & Omit<React.ComponentProps<typeof Modal>, "show" | "onHide">;

type ConfirmModalProps<T> = ReactConfirmProps & ConfirmModalOptions<T>;

export class ConfirmModal extends React.Component<ConfirmModalProps<any>, { data: any }> {
  readonly state: { data: any } = { data: this.props.data };

  componentDidUpdate = (prevProps: ConfirmModalProps<any>) => {
    if (prevProps.data !== this.props.data) {
      this.setState({ data: this.props.data });
    }
  }

  render() {
    const { className, title, confirmation, show, proceed, cancel, dismiss, render, renderInner, okBtn, cancelBtn, okText, cancelText, cancelIfBackdrop, okVariant, cancelVariant, ...props } = this.props;

    // @ts-ignore
    const okFn = (data?: any) => proceed(data);
    const cancelFn = () => cancel();

    return <Modal
      show={show}
      onHide={cancelFn}
      backdrop={cancelIfBackdrop ? undefined : "static"}
      centered
      { ...props }
    >
      {
        title ? <Modal.Header> <Modal.Title> { title } </Modal.Title> </Modal.Header> : undefined
      }
      {
        render ? render(okFn, cancelFn) : <React.Fragment>
          <Modal.Body> 
            {
              renderInner ? renderInner(this.state.data, (data: any) => this.setState({ data: data }), () => okFn(this.state.data), cancelFn)
                : (confirmation || "Are you sure?")
            }
          </Modal.Body>
          <Modal.Footer>
            {
              okBtn ? <Button {...okBtn} onClick={() => okFn(this.state.data)} /> : 
              <Button
                onClick={() => okFn(this.state.data)}
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
  let data: any = await confirmBackend({ confirmation, ...(options || {}) });
  return data as T;
}

export default async function confirmBool(confirmation: ReactConfirmProps["confirmation"], options?: ConfirmModalOptions<{}>) {
  try {
    await confirmBackend({ confirmation, title: "Are you sure?", okText: "Yes", okVariant: "danger", ...(options || {}) });
    return true;
  } catch {
    return false;
  }
}

export async function withConfirm(fn: () => void, confirmation?: ReactConfirmProps["confirmation"], options?: ConfirmModalOptions<{}>) {
  if (await confirmBool(confirmation || "Are you sure?", options)) {
    fn();
  }
}