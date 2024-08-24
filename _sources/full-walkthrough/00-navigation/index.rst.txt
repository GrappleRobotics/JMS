JMS Navigation
==============

When browsing to the JMS UI (http://10.0.100.10/), you will be greeted with a page resembling the following.

.. note::
  On your first login, it will ask for an FTA password. Set this to something you will remember, since this is the admin user for JMS!
    
.. image:: imgs/peripherals.png

In the main view, you will find a tile for everything the current user is authorised to access. If you're not the admin user, you may only have access to a few of these.

On the peripherals of the screen, you will find a set of buttons and indicators that represent the current state of JMS. These elements are:

- **A**: The Emergency Stop button. The |estop| button is only present for users which have permission to E-Stop the field, usually referees and the FTA/FTAAs. Clicking this button opens a large dialog in which you can confirm or cancel your choice to Emergency Stop the field.

.. warning::
  Clicking the |estop| button will log who clicked it!

.. image:: imgs/estop-confirm.png

- **B**: The current Arena State. Accompanied by a change in top bar colour, this indicator will change according to the current state of the arena: Idle, Prestart (Working), Prestart (Ready), Emergency Stop, Match ARMed, Auto, Teleop, and Match Complete. In Match Play states (Auto, Teleop), it is accompanied by a countdown timer.

.. image:: imgs/top-bar-estop.png
.. image:: imgs/top-bar-prestart-ready.png
.. image:: imgs/top-bar-armed.png
.. image:: imgs/top-bar-auto.png

- **C**: The current user (if logged in), and the login/logout button.

- **D**: Container Status. JMS is deployed as a set of Docker Containers, each of which with its own purpose and function. A green indicator implies the container is up and running, whilst red usually denotes that it has been stopped or has crashed. Usually, this is only relevant to the FTA, but if you see any of these as red, make sure to alert your FTA!

- **E**: Schedule Clock. When a schedule is loaded, this will tell you how far ahead or behind the field is running. 

- **F**: Refresh button, for if the UI stops responding on a mobile device.
