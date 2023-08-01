"use client"

import { withPermission } from "@/app/support/permissions"
import { faInfoCircle } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import React from "react";

export default withPermission("Admin", function EventWizardUsers() {
  return <React.Fragment>
    <h3> Team Management </h3>
    <p className="text-muted"> 
      <FontAwesomeIcon icon={faInfoCircle} /> &nbsp; 
      After the Match Schedule is generated, this list can no longer be changed. You need at least 6 teams to generate a schedule.
    </p>

    
  </React.Fragment>
});