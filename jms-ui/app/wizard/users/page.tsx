"use client";
import { confirmModal, withConfirm } from "@/app/components/Confirm";
import { useErrors } from "@/app/support/errors";
import { PERMISSIONS, withPermission } from "@/app/support/permissions";
import { useWebsocket } from "@/app/support/ws-component";
import { Permission, User } from "@/app/ws-schema";
import { faTrash } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import React, { useEffect, useState } from "react";
import { Button, Card, Form, InputGroup, Table } from "react-bootstrap";
import { Typeahead } from "react-bootstrap-typeahead";
import update from "immutability-helper";
import JmsWebsocket from "@/app/support/ws";

async function newUserModal(call: JmsWebsocket["call"]) {
  let details = await confirmModal("", {
    title: "New User",
    data: {
      username: "",
      realname: "",
      permissions: [],
      tokens: [],
      pin_is_numeric: false
    } as User,
    renderInner: (data, onUpdate) => <React.Fragment>
      <InputGroup className="m-2">
        <InputGroup.Text>Username</InputGroup.Text>
        <Form.Control
          type="text"
          value={data.username}
          onChange={e => onUpdate(update(data, { username: { $set: e.target.value.trim() } }))}
        />
      </InputGroup>

      <InputGroup className="m-2">
        <InputGroup.Text>Real Name</InputGroup.Text>
        <Form.Control
          type="text"
          value={data.realname}
          onChange={e => onUpdate(update(data, { realname: { $set: e.target.value.trim() } }))}
        />
      </InputGroup>

      <InputGroup className="m-2">
        <InputGroup.Text>Permissions</InputGroup.Text>
        <Typeahead
          id={`role-typeahead-modal`}
          multiple
          options={Object.keys(PERMISSIONS)}
          selected={data.permissions}
          onChange={(perms) => onUpdate(update(data, { permissions: { $set: perms as Permission[] } }))}
        />
      </InputGroup>
    </React.Fragment>,
  });

  call<"user/modify_user">("user/modify_user", { user: details })
    .catch(e => alert(e));
}

export default withPermission(["Admin"], function EventWizardUsers() {
  const [ users, setUsers ] = useState<User[]>([]);
  const { call } = useWebsocket();

  const { addError } = useErrors();

  const refreshUsers = () => {
    call<"user/users">("user/users", null).then(setUsers).catch(addError);
  };

  useEffect(() => {
    refreshUsers();
  }, []);

  return <React.Fragment>
    <h3> User Management </h3>

    <Table className="my-4" striped hover>
      <thead>
        <tr>
          <th> Username </th>
          <th> Real Name </th>
          <th> Permissions </th>
          <th> Actions </th>
        </tr>
      </thead>
      <tbody>
        {
          users.map(user => <tr>
            <td> { user.username } </td>
            <td> { user.realname } </td>
            <td>
              <Typeahead
                id={`role-typeahead-${user.username}`}
                multiple
                options={Object.keys(PERMISSIONS)}
                selected={user.permissions}
                onChange={(perms) => {
                  call<"user/modify_user">("user/modify_user", { user: { ...user, permissions: perms } as any })
                    .then(refreshUsers)
                    .catch(addError);
                }}
                size="sm"
              />
            </td>
            <td>
              <Button
                size="sm"
                variant="danger"
                disabled={user.permissions.includes("Admin")}
                onClick={() => withConfirm(() => call<"user/delete_user">("user/delete_user", { user_id: user.username }).then(refreshUsers))}
              >
                <FontAwesomeIcon icon={faTrash} />
              </Button> &nbsp;
              <Button
                size="sm"
                variant="info"
                onClick={() => withConfirm(() => call<"user/modify_user">("user/modify_user", { user: { ...user, pin_hash: null } }).catch(addError))}
              >
                Reset PIN
              </Button> &nbsp;
              <Button
                size="sm"
                variant="warning"
                onClick={() => withConfirm(() => call<"user/modify_user">("user/modify_user", { user: { ...user, tokens: [] } }))}
              >
                Invalidate Tokens
              </Button>
            </td>
          </tr>)
        }
      </tbody>
    </Table>

    <Button variant="success" onClick={() => newUserModal(call).then(refreshUsers)}>
      Add User
    </Button>
  </React.Fragment>
})