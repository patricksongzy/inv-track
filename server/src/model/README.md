## model
* contains the data model
* note there is some duplication between models representing query results and models representing insertable items
  * this is purposeful since often times the two do not line up
  * a more elegant solution might be to create some macros (ie use AOP instead)
