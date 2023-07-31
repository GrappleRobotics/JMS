"use client";
import { useErrors } from "@/app/support/errors";
import { PERMISSIONS } from "@/app/support/permissions";
import { useWebsocket } from "@/app/support/ws-component";
import { User } from "@/app/ws-schema";
import { faTrash } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import React, { useEffect, useState } from "react";
import { Button, Card, Form, Table } from "react-bootstrap";
import { Typeahead } from "react-bootstrap-typeahead";

export default function EventWizardUsers() {
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
    <Card>
      <Card.Body>
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
                  <Button size="sm" variant="danger" disabled={user.permissions.includes("Admin")}>
                    <FontAwesomeIcon icon={faTrash} />
                  </Button> &nbsp;
                  <Button size="sm" variant="info" onClick={() => call<"user/modify_user">("user/modify_user", { user: { ...user, pin_hash: null } }).catch(addError)}>
                    Reset PIN
                  </Button> &nbsp;
                  <Button size="sm" variant="warning" onClick={() => call<"user/modify_user">("user/modify_user", { user: { ...user, tokens: [] } })}>
                    Invalidate Tokens
                  </Button>
                </td>
              </tr>)
            }
          </tbody>
        </Table>

        <Button variant="success">
          Add User
        </Button>
      </Card.Body>
    </Card>
  </React.Fragment>
}